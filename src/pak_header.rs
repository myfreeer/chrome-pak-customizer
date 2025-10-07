#![allow(unaligned_references)]

use std::mem::size_of;

use crate::pak_def::{PakAlias, PakBase, PakEntry, serialize};
use crate::pak_error::PakError;
use crate::pak_index::NumDigits;

pub trait PakHeader : PakBase {
    fn read_version(&self) -> u32;
    fn write_version(&mut self, version: u32);
    fn read_encoding(&self) -> u8;
    fn write_encoding(&mut self, encoding: u8);
    fn read_resource_count(&self) -> u32;
    fn write_resource_count(&mut self, resource_count: u32);
    fn read_alias_count(&self) -> u32;
    fn write_alias_count(&mut self, alias_count: u32);
    fn size(&self) -> usize;
    fn resource_size(&self) -> usize;
    fn alias_size(&self) -> usize;
    fn alias_offset(&self) -> usize;
}

const PAK_VERSION_SIZE: usize = size_of::<u32>();
pub const PAK_VERSION_V5: u32 = 5;
pub const PAK_VERSION_V4: u32 = 4;

// v5 header:
// uint32(version), uint8(encoding), 3 bytes padding,
// uint16(resource_count), uint16(alias_count)
#[repr(packed(1))]
pub struct PakHeaderV5<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> {
    pub version: u32,
    pub encoding: u8,
    pub  _padding: [u8; 3],
    pub resource_count: T,
    pub alias_count: T,
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakBase for PakHeaderV5<T> {
    fn from_buf(buf: &[u8]) -> Result<&Self, PakError> {
        if buf.len() < size_of::<Self>() {
            return Err(PakError::V5HeaderSizeNotEnough(
                buf.len(), size_of::<Self>(),
            ));
        }
        let p: * mut Self = buf.as_ptr() as * mut Self;
        let header: &Self = unsafe { &*p };
        if header.version != PAK_VERSION_V5 {
            return Err(PakError::VersionMisMatch(
                header.version, PAK_VERSION_V5));
        }
        Ok(header)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    #[inline]
    fn new() -> Self {
        PakHeaderV5 {
            version: PAK_VERSION_V5,
            encoding: 0,
            _padding: [0, 0, 0],
            resource_count: Default::default(),
            alias_count: Default::default(),
        }
    }
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakHeader for PakHeaderV5<T> {
    #[inline]
    fn read_version(&self) -> u32 {
        self.version
    }

    #[inline]
    fn write_version(&mut self, version: u32) {
        self.version = version
    }

    #[inline]
    fn read_encoding(&self) -> u8 {
        self.encoding
    }

    #[inline]
    fn write_encoding(&mut self, encoding: u8) {
        self.encoding = encoding
    }

    #[inline]
    fn read_resource_count(&self) -> u32 {
        self.resource_count.into()
    }

    #[inline]
    fn write_resource_count(&mut self, resource_count: u32) {
        self.resource_count = resource_count.try_into().unwrap_or_default()
    }

    #[inline]
    fn read_alias_count(&self) -> u32 {
        self.alias_count.into()
    }

    #[inline]
    fn write_alias_count(&mut self, alias_count: u32) {
        self.alias_count = alias_count.try_into().unwrap_or_default()
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<Self>()
    }

    #[inline]
    fn resource_size(&self) -> usize {
        let resource_count: u32 = self.resource_count.into();
        ((resource_count as usize) + 1) * size_of::<PakEntry<T>>()
    }

    #[inline]
    fn alias_size(&self) -> usize {
        let alias_count: u32 = self.alias_count.into();
        (alias_count as usize) * size_of::<PakAlias<T>>()
    }

    #[inline]
    fn alias_offset(&self) -> usize {
       self.size() + self.resource_size()
    }
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> Default for PakHeaderV5<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// v4 header: uint32(version), uint32(resource_count), uint8(encoding)
#[repr(packed(1))]
pub struct PakHeaderV4 {
    pub version: u32,
    pub resource_count: u32,
    pub encoding: u8,
}

impl PakBase for PakHeaderV4 {
    fn from_buf(buf: &[u8]) -> Result<&PakHeaderV4, PakError> {
        if buf.len() < size_of::<PakHeaderV4>() {
            return Err(PakError::V4HeaderSizeNotEnough(
                buf.len(), size_of::<PakHeaderV4>()));
        }
        let p: * mut PakHeaderV4 = buf.as_ptr() as * mut PakHeaderV4;
        let header: &PakHeaderV4 = unsafe { &*p };
        if header.version != PAK_VERSION_V4 {
            return Err(PakError::VersionMisMatch(
                header.version, PAK_VERSION_V4));
        }
        Ok(header)
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    #[inline]
    fn new() -> PakHeaderV4 {
        PakHeaderV4 {
            version: PAK_VERSION_V4,
            resource_count: 0,
            encoding: 0,
        }
    }
}

impl PakHeader for PakHeaderV4 {
    #[inline]
    fn read_version(&self) -> u32 {
        self.version
    }

    #[inline]
    fn write_version(&mut self, version: u32) {
        self.version = version
    }

    #[inline]
    fn read_encoding(&self) -> u8 {
        self.encoding
    }

    #[inline]
    fn write_encoding(&mut self, encoding: u8) {
        self.encoding = encoding
    }

    #[inline]
    fn read_resource_count(&self) -> u32 {
        self.resource_count
    }

    #[inline]
    fn write_resource_count(&mut self, resource_count: u32) {
        self.resource_count = resource_count
    }

    #[inline]
    fn read_alias_count(&self) -> u32 {
        0
    }

    #[inline]
    fn write_alias_count(&mut self, _alias_count: u32) {
        unimplemented!("Not supported")
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<PakHeaderV4>()
    }

    #[inline]
    fn resource_size(&self) -> usize {
        ((self.resource_count as usize) + 1) * size_of::<PakEntry<u16>>()
    }

    #[inline]
    fn alias_size(&self) -> usize {
        0
    }

    #[inline]
    fn alias_offset(&self) -> usize {
        unimplemented!("Not supported")
    }
}

impl Default for PakHeaderV4 {
    #[inline]
    fn default() -> Self {
        PakHeaderV4::new()
    }
}

#[inline]
pub fn pak_get_version(buf: &[u8]) -> Result<u32, PakError> {
    if buf.len() < PAK_VERSION_SIZE {
        return Err(PakError::VersionSizeNotEnough(
            buf.len(), PAK_VERSION_SIZE));
    }
    Ok(u32::from_le_bytes(buf[..4].try_into().unwrap()))
}

pub fn pak_read_header(buf: &[u8], edge_v5: bool) -> Result<& dyn PakHeader, PakError> {
    let version = pak_get_version(buf)?;
    match version {
        PAK_VERSION_V5 => {
            if edge_v5 {
                let header = PakHeaderV5::<u32>::from_buf(buf)?;
                return Ok(header);
            }
            let header = PakHeaderV5::<u16>::from_buf(buf)?;
            Ok(header)
        }
        PAK_VERSION_V4 => {
            let header =  PakHeaderV4::from_buf(buf)?;
            Ok(header)
        }
        version => Err(PakError::UnsupportedVersion(version))
    }
}
