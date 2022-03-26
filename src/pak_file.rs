use std::mem::size_of;

use crate::pak_def::{PakBaseOffset, PakEntry};
use crate::pak_error::PakError;
use crate::pak_header::PakHeader;

pub struct PakFile<'a> {
    pub id: u16,
    pub buf: &'a [u8],
}

pub fn pak_parse_files<'a>(header: &'a dyn PakHeader, buf: &'a [u8])
                           -> Result<Vec<PakFile<'a>>, PakError> {
    let mut resource_count = header.read_resource_count();
    if resource_count == 0 {
        return Err(PakError::PakZeroResourceCount);
    }
    let mut vec: Vec<PakFile<'a>> = Vec::with_capacity(resource_count as usize);
    let mut header_offset = header.size();
    resource_count += 1;
    let mut last_entry: Option<&PakEntry> = None;
    for _i in 0..resource_count {
        let entry = PakEntry::from_buf_offset(buf, header_offset)?;
        match last_entry {
            None => {}
            Some(last_entry) => {
                let begin_offset = last_entry.offset as usize;
                let end_offset = entry.offset as usize;
                let buf_slice = &buf[begin_offset..end_offset];
                let file = PakFile {
                    id: last_entry.resource_id,
                    buf: buf_slice
                };
                vec.push(file);
            }
        }
        last_entry = Some(entry);
        header_offset += size_of::<PakEntry>();
    }
    Ok(vec)
}