#![allow(unaligned_references)]

use std::mem::size_of;

use byteorder::ByteOrder;
use type_layout::TypeLayout;

use crate::pak_error::PakError;

pub trait PakBase {
    fn from_buf(buf: &[u8]) -> Result<Self, PakError> where Self: Sized;
    fn as_bytes(&self) -> &[u8];
    fn new() -> Self where Self: Sized;
}

pub trait PakHeader : PakBase {
    fn read_version(&self) -> u32;
    fn write_version(&mut self, version: u32);
    fn read_encoding(&self) -> u8;
    fn write_encoding(&mut self, encoding: u8);
    fn read_resource_count(&self) -> u32;
    fn write_resource_count(&mut self, resource_count: u32);
    fn read_alias_count(&self) -> u16;
    fn write_alias_count(&mut self, alias_count: u16);
}


pub unsafe fn serialize<T: Sized>(src: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (src as *const T) as *const u8,
        ::std::mem::size_of::<T>())
}

const PAK_VERSION_SIZE: usize = 4;
const PAK_VERSION_V5: u32 = 5;
const PAK_VERSION_V4: u32 = 4;

// v5 header:
// uint32(version), uint8(encoding), 3 bytes padding,
// uint16(resource_count), uint16(alias_count)
#[derive(TypeLayout)]
#[repr(packed(1))]
pub struct PakHeaderV5 {
    version: u32,
    encoding: u8,
    _padding: [u8; 3],
    resource_count: u16,
    alias_count: u16,
}

impl PakBase for PakHeaderV5 {
    fn from_buf(buf: &[u8]) -> Result<PakHeaderV5, PakError> {
        if buf.len() < size_of::<PakHeaderV5>() {
            return Err(PakError::V5HeaderSizeNotEnough(
                buf.len(), size_of::<PakHeaderV5>(),
            ));
        }
        let header: PakHeaderV5 = unsafe {
            std::ptr::read(buf.as_ptr() as *const _)
        };
        if header.version != PAK_VERSION_V5 {
            return Err(PakError::VersionMisMatch(
                header.version, PAK_VERSION_V5));
        }
        Ok(header)
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

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
    fn read_version(&self) -> u32 {
        self.version
    }

    fn write_version(&mut self, version: u32) {
        self.version = version
    }

    fn read_encoding(&self) -> u8 {
        self.encoding
    }

    fn write_encoding(&mut self, encoding: u8) {
        self.encoding = encoding
    }

    fn read_resource_count(&self) -> u32 {
        self.resource_count as u32
    }

    fn write_resource_count(&mut self, resource_count: u32) {
        self.resource_count = resource_count as u16
    }

    fn read_alias_count(&self) -> u16 {
        self.alias_count
    }

    fn write_alias_count(&mut self, alias_count: u16) {
        self.alias_count = alias_count
    }
}

// v4 header: uint32(version), uint32(resource_count), uint8(encoding)
#[derive(TypeLayout)]
#[repr(packed(1))]
#[allow(unaligned_references)]
pub struct PakHeaderV4 {
    version: u32,
    resource_count: u32,
    encoding: u8,
}

impl PakBase for PakHeaderV4 {
    fn from_buf(buf: &[u8]) -> Result<PakHeaderV4, PakError> {
        if buf.len() < size_of::<PakHeaderV4>() {
            return Err(PakError::V4HeaderSizeNotEnough(
                buf.len(), size_of::<PakHeaderV4>()));
        }
        let header: PakHeaderV4 = unsafe {
            std::ptr::read(buf.as_ptr() as *const _)
        };
        if header.version != PAK_VERSION_V4 {
            return Err(PakError::VersionMisMatch(
                header.version, PAK_VERSION_V4));
        }
        Ok(header)
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { serialize(self) }
    }

    fn new() -> PakHeaderV4 {
        PakHeaderV4 {
            version: PAK_VERSION_V4,
            resource_count: 0,
            encoding: 0,
        }
    }
}

impl PakHeader for PakHeaderV4 {
    fn read_version(&self) -> u32 {
        self.version
    }

    fn write_version(&mut self, version: u32) {
        self.version = version
    }

    fn read_encoding(&self) -> u8 {
        self.encoding
    }

    fn write_encoding(&mut self, encoding: u8) {
        self.encoding = encoding
    }

    fn read_resource_count(&self) -> u32 {
        self.resource_count
    }

    fn write_resource_count(&mut self, resource_count: u32) {
        self.resource_count = resource_count
    }

    fn read_alias_count(&self) -> u16 {
        unimplemented!("Not supported")
    }

    fn write_alias_count(&mut self, _alias_count: u16) {
        unimplemented!("Not supported")
    }
}

pub fn pak_get_version(buf: &[u8]) -> Result<u32, PakError> {
    if buf.len() < PAK_VERSION_SIZE {
        return Err(PakError::VersionSizeNotEnough(
            buf.len(), PAK_VERSION_SIZE));
    }
    Ok(byteorder::LittleEndian::read_u32(buf))
}

pub fn pak_read_header(buf: &[u8]) -> Result<Box<dyn PakHeader>, PakError> {
    let result = pak_get_version(buf);
    match result {
        Ok(version) => match version {
            PAK_VERSION_V5 => {
                let result = PakHeaderV5::from_buf(buf);
                match result {
                    Ok(header) => Ok(Box::new(header)),
                    Err(err) => Err(err)
                }
            }
            PAK_VERSION_V4 => {
                let result = PakHeaderV4::from_buf(buf);
                match result {
                    Ok(header) => Ok(Box::new(header)),
                    Err(err) => Err(err)
                }
            }
            version => Err(PakError::UnsupportedVersion(version))
        }
        Err(err) => Err(err)
    }
}
