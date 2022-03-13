#![allow(unaligned_references)]

use std::mem::size_of;

use crate::pak_def::{PakBase, PakEntry, serialize};
use crate::pak_error::PakError;

pub trait PakHeader : PakBase {
    fn read_version(&self) -> u32;
    fn write_version(&mut self, version: u32);
    fn read_encoding(&self) -> u8;
    fn write_encoding(&mut self, encoding: u8);
    fn read_resource_count(&self) -> u32;
    fn write_resource_count(&mut self, resource_count: u32);
    fn read_alias_count(&self) -> u16;
    fn write_alias_count(&mut self, alias_count: u16);
    fn size(&self) -> usize;
    fn alias_offset(&self) -> usize;
}

const PAK_VERSION_SIZE: usize = size_of::<u32>();
pub const PAK_VERSION_V5: u32 = 5;
pub const PAK_VERSION_V4: u32 = 4;

// v5 header:
// uint32(version), uint8(encoding), 3 bytes padding,
// uint16(resource_count), uint16(alias_count)
#[repr(packed(1))]
#[derive(Debug)]
pub struct PakHeaderV5 {
    pub version: u32,
    pub encoding: u8,
    pub  _padding: [u8; 3],
    pub resource_count: u16,
    pub alias_count: u16,
}

impl PakBase for PakHeaderV5 {
    fn from_buf(buf: &[u8]) -> Result<&PakHeaderV5, PakError> {
        if buf.len() < size_of::<PakHeaderV5>() {
            return Err(PakError::V5HeaderSizeNotEnough(
                buf.len(), size_of::<PakHeaderV5>(),
            ));
        }
        let p: * mut PakHeaderV5 = buf.as_ptr() as * mut PakHeaderV5;
        let header: &PakHeaderV5 = unsafe { &*p };
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
    fn new() -> PakHeaderV5 {
        PakHeaderV5 {
            version: PAK_VERSION_V5,
            encoding: 0,
            _padding: [0, 0, 0],
            resource_count: 0,
            alias_count: 0,
        }
    }
}

impl PakHeader for PakHeaderV5 {
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
        self.resource_count as u32
    }

    #[inline]
    fn write_resource_count(&mut self, resource_count: u32) {
        self.resource_count = resource_count as u16
    }

    #[inline]
    fn read_alias_count(&self) -> u16 {
        self.alias_count
    }

    #[inline]
    fn write_alias_count(&mut self, alias_count: u16) {
        self.alias_count = alias_count
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<PakHeaderV5>()
    }

    #[inline]
    fn alias_offset(&self) -> usize {
       self.size() + ((self.resource_count as usize) + 1) * size_of::<PakEntry>()
    }
}

impl Default for PakHeaderV5 {
    #[inline]
    fn default() -> Self {
        PakHeaderV5::new()
    }
}

// v4 header: uint32(version), uint32(resource_count), uint8(encoding)
#[repr(packed(1))]
#[derive(Debug)]
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
    fn read_alias_count(&self) -> u16 {
        0
    }

    #[inline]
    fn write_alias_count(&mut self, _alias_count: u16) {
        unimplemented!("Not supported")
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<PakHeaderV4>()
    }

    #[inline]
    fn alias_offset(&self) -> usize {
        self.size()
    }
}

impl Default for PakHeaderV4 {
    #[inline]
    fn default() -> Self {
        PakHeaderV4::new()
    }
}

pub fn pak_get_version(buf: &[u8]) -> Result<u32, PakError> {
    if buf.len() < PAK_VERSION_SIZE {
        return Err(PakError::VersionSizeNotEnough(
            buf.len(), PAK_VERSION_SIZE));
    }
    Ok(u32::from_le_bytes(buf[..4].try_into().unwrap()))
}

pub fn pak_read_header(buf: &[u8]) -> Result<& dyn PakHeader, PakError> {
    let result = pak_get_version(buf);
    match result {
        Ok(version) => match version {
            PAK_VERSION_V5 => {
                let result = PakHeaderV5::from_buf(buf);
                match result {
                    Ok(header) => Ok(header),
                    Err(err) => Err(err)
                }
            }
            PAK_VERSION_V4 => {
                let result = PakHeaderV4::from_buf(buf);
                match result {
                    Ok(header) => Ok(header),
                    Err(err) => Err(err)
                }
            }
            version => Err(PakError::UnsupportedVersion(version))
        }
        Err(err) => Err(err)
    }
}
