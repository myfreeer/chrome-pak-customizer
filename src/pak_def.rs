#![allow(unaligned_references)]

use std::mem::size_of;

use crate::pak_error::PakError;
use crate::pak_header::PakHeader;
use crate::pak_index::NumDigits;

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
#[derive(Default)]
pub struct PakEntry<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> {
    pub resource_id: T,
    pub offset: u32,
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakBase for PakEntry<T> {
    #[inline]
    fn from_buf(buf: &[u8]) -> Result<&Self, PakError> {
        Self::from_buf_offset(buf, 0)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    #[inline]
    fn new() -> Self {
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

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakBaseOffset for PakEntry<T> {
    #[inline]
    fn from_buf_offset(buf: &[u8], offset: usize) -> Result<&Self, PakError> {
        from_buf_offset(buf, offset)
    }
}

// Alias: uint16_t resourceId; uint16_t entry_index;
#[repr(packed(1))]
#[derive(Default)]
pub struct PakAlias<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> {
    pub resource_id: T,
    pub entry_index: T,
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakBase for PakAlias<T> {
    #[inline]
    fn from_buf(buf: &[u8]) -> Result<&Self, PakError> {
        PakAlias::from_buf_offset(buf, 0)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    #[inline]
    fn new() -> Self {
        PakAlias::default()
    }
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakBaseOffset for PakAlias<T> {
    #[inline]
    fn from_buf_offset(buf: &[u8], offset: usize) -> Result<&Self, PakError> {
        from_buf_offset(buf, offset)
    }
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakAlias<T> {
    #[inline]
    pub fn serialize_slice(alias_slice: &[Self], alias_size: usize) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (alias_slice as *const [Self]) as *const u8,
                alias_size)
        }
    }

    #[inline(always)]
    pub fn read_resource_id(&self) -> u32 {
        self.resource_id.into()
    }

    #[inline(always)]
    pub fn read_entry_index(&self) -> u32 {
        self.entry_index.into()
    }
}

pub fn pak_parse_alias<'a, T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    header: &dyn PakHeader,
    buf: &'a [u8]
) -> Result<&'a [PakAlias<T>], PakError> {
    let alias_count = header.read_alias_count();
    // no resource is bad, no alias is ok
    if alias_count == 0 {
        return Ok(&[]);
    }
    let offset = header.alias_offset();
    pak_read_alias_slice(buf, offset, alias_count)
}

pub fn pak_read_alias_slice<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    buf: &[u8],
    offset: usize,
    alias_count: u32
) -> Result<&[PakAlias<T>], PakError> {
    // no resource is bad, no alias is ok
    if alias_count == 0 {
        return Ok(&[]);
    }
    let len = buf.len();
    if len < offset {
        return Err(PakError::PakAliasOffsetOverflow(len, offset));
    }
    let remaining_size = len - offset;
    let required_size = size_of::<PakAlias<T>>() * (alias_count as usize);
    if remaining_size < required_size {
        return Err(PakError::PakAliasSizeNotEnough(
            remaining_size, required_size));
    }
    Ok(unsafe {
        let p: *mut PakAlias<T> = buf.as_ptr().add(offset) as *mut PakAlias<T>;
        std::slice::from_raw_parts(p, alias_count as usize)
    })
}
