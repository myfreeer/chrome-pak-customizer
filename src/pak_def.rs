#![allow(unaligned_references)]

use std::mem::size_of;

use crate::pak_error::PakError;
use crate::pak_header::PakHeader;

pub trait PakBase {
    fn from_buf(buf: &[u8]) -> Result<&Self, PakError> where Self: Sized;
    fn as_bytes(&self) -> &[u8];
    fn new() -> Self where Self: Sized;
}

pub trait PakBaseOffset: PakBase {
    fn from_buf_offset(buf: &[u8], offset: usize)
                       -> Result<&Self, PakError> where Self: Sized;
}

#[inline]
pub unsafe fn serialize<T: Sized>(src: &T) -> &[u8] {
    std::slice::from_raw_parts(
        (src as *const T) as *const u8,
        ::std::mem::size_of::<T>())
}

// Entry: uint16_t resourceId; uint32_t offset;
#[repr(packed(1))]
#[derive(Default, Debug)]
pub struct PakEntry {
    pub resource_id: u16,
    pub offset: u32,
}

impl PakBase for PakEntry {
    fn from_buf(buf: &[u8]) -> Result<&PakEntry, PakError> {
        PakEntry::from_buf_offset(buf, 0)
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    fn new() -> PakEntry {
        PakEntry::default()
    }
}

fn from_buf_offset<T: Sized>(buf: &[u8], offset: usize) -> Result<&T, PakError> {
    let len = buf.len();
    if len < offset {
        return Err(PakError::PakEntryOrAliasOffsetOverflow(len, offset));
    }
    let remaining_size = len - offset;
    let required_size = size_of::<T>();
    if remaining_size < required_size {
        return Err(PakError::PakEntryOrAliasSizeNotEnough(
            remaining_size, required_size));
    }
    Ok(unsafe {
        let p: *mut T = buf.as_ptr().add(offset) as *mut T;
        &*(p)
    })
}

impl PakBaseOffset for PakEntry {
    fn from_buf_offset(buf: &[u8], offset: usize) -> Result<&PakEntry, PakError> {
        from_buf_offset(buf, offset)
    }
}

// Alias: uint16_t resourceId; uint16_t entry_index;
#[repr(packed(1))]
#[derive(Default, Debug)]
pub struct PakAlias {
    pub resource_id: u16,
    pub entry_index: u16,
}

impl PakBase for PakAlias {
    fn from_buf(buf: &[u8]) -> Result<&PakAlias, PakError> {
        PakAlias::from_buf_offset(buf, 0)
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    fn new() -> PakAlias {
        PakAlias::default()
    }
}

impl PakBaseOffset for PakAlias {
    fn from_buf_offset(buf: &[u8], offset: usize) -> Result<&PakAlias, PakError> {
        from_buf_offset(buf, offset)
    }
}

pub fn pak_parse_alias(header: &dyn PakHeader, buf: &[u8])
                       -> Result<&[PakAlias], PakError> {
    let alias_count = header.read_alias_count();
    let offset = header.alias_offset();
    pak_read_alias_slice(buf, offset, alias_count)
}

pub fn pak_read_alias_slice(buf: &[u8], offset: usize, alias_count: u16)
                            -> Result<&[PakAlias], PakError> {
    // no resource is bad, no alias is ok
    if alias_count == 0 {
        return Ok(&[]);
    }
    let len = buf.len();
    if len < offset {
        return Err(PakError::PakAliasOffsetOverflow(len, offset));
    }
    let remaining_size = len - offset;
    let required_size = size_of::<PakAlias>() * (alias_count as usize);
    if remaining_size < required_size {
        return Err(PakError::PakAliasSizeNotEnough(
            remaining_size, required_size));
    }
    Ok(unsafe {
        let p: *mut PakAlias = buf.as_ptr().add(offset) as *mut PakAlias;
        std::slice::from_raw_parts(p, alias_count as usize)
    })
}
