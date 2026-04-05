use std::mem::size_of;

use crate::pak_def::{PakBaseOffset, PakEntry, checked_add_usize};
use crate::pak_error::PakError;
use crate::pak_header::PakHeader;
use crate::pak_index::NumDigits;

pub struct PakFile<'a> {
    pub id: u32,
    pub buf: &'a [u8],
}

pub fn pak_parse_files<'a, T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    header: &'a dyn PakHeader,
    buf: &'a [u8]
) -> Result<Vec<PakFile<'a>>, PakError> {
    let mut resource_count = header.read_resource_count();
    if resource_count == 0 {
        return Err(PakError::PakZeroResourceCount);
    }
    let mut vec: Vec<PakFile<'a>> = Vec::with_capacity(resource_count as usize);
    let mut header_offset = header.size();
    resource_count = resource_count
        .checked_add(1)
        .ok_or(PakError::PakArithmeticOverflow("resource entry count"))?;
    let mut last_entry: Option<&PakEntry<T>> = None;
    for _i in 0..resource_count {
        let entry = PakEntry::<T>::from_buf_offset(buf, header_offset)?;
        if let Some(last_entry) = last_entry {
            let begin_offset = last_entry.read_offset() as usize;
            let end_offset = entry.read_offset() as usize;
            if begin_offset > end_offset {
                return Err(PakError::PakPackResourceInvalidOffset(
                    last_entry.read_offset(),
                    entry.read_offset(),
                ));
            }
            if end_offset > buf.len() {
                return Err(PakError::PakPackResourceOffsetOverflow(
                    entry.read_offset(),
                    buf.len(),
                ));
            }
            let buf_slice = &buf[begin_offset..end_offset];
            let file = PakFile {
                id: last_entry.read_resource_id(),
                buf: buf_slice
            };
            vec.push(file);
        }
        last_entry = Some(entry);
        header_offset = checked_add_usize(
            header_offset,
            size_of::<PakEntry<T>>(),
            "resource table offset",
        )?;
    }
    Ok(vec)
}
