use std::borrow::{Borrow, BorrowMut};
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::FromStr;

use ini_core::Item;

use crate::pak_def::{PakAlias, PakBase};
use crate::pak_error::PakError;
use crate::pak_error::PakError::PakIndexParseError;
use crate::pak_header::{PAK_VERSION_V4, PAK_VERSION_V5, PakHeader, PakHeaderV4, PakHeaderV5};
use crate::pak_index::PakIndexStatus::Resource;

// TODO: compression
pub struct PakIndexEntry {
    pub resource_id: u16,
    pub file_name: String,
}

pub struct PakIndexRef<'a> {
    pub header: &'a dyn PakHeader,
    pub entry_slice: &'a [PakIndexEntry],
    pub alias_slice: &'a [PakAlias],
}

static NUMBER_DECIMAL_U16: [u16; 5] = [
    10, 100, 1000, 10000, u16::MAX
];

fn number_digit_count_u16(num: u16) -> usize {
    let mut count: usize = 1;
    for x in NUMBER_DECIMAL_U16 {
        if num < x {
            return count;
        }
        count += 1;
    }
    count
}

pub const PAK_INDEX_GLOBAL_TAG: &str = "Global";
pub const PAK_INDEX_RES_TAG: &str = "Resources";
pub const PAK_INDEX_ALIAS_TAG: &str = "Alias";
pub const PAK_INDEX_GLOBAL_VERSION: &str = "version";
pub const PAK_INDEX_GLOBAL_ENCODING: &str = "encoding";
pub const PAK_INDEX_TAG_END: &str = "]\r\n";
pub const PAK_INDEX_CRLF: &str = "\r\n";

#[derive(Clone, Copy)]
pub enum PakIndexStatus {
    Init,
    Global,
    Resource,
    Alias,
}

// TODO: pak index <-> pak header + buf
impl PakIndexRef<'_> {
    fn calc_ini_byte_size(&self) -> usize {
        // 12: []\r\n * 2 + \r\n\r\n
        let mut buf_size: usize =
            PAK_INDEX_GLOBAL_TAG.len() + PAK_INDEX_RES_TAG.len() + 12;
        // 4: = \r\n + version number
        buf_size += PAK_INDEX_GLOBAL_VERSION.len() + 4;
        // 4: = \r\n + encoding number
        buf_size += PAK_INDEX_GLOBAL_ENCODING.len() + 4;
        if !self.alias_slice.is_empty() {
            // 8: \r\n\r\n + []\r\n
            buf_size += PAK_INDEX_ALIAS_TAG.len() + 8;
        }
        for entry in self.entry_slice {
            // 3: =\r\n
            buf_size += number_digit_count_u16(entry.resource_id) + 3;
            buf_size += entry.file_name.len();
        }
        for alias in self.alias_slice {
            // 3: =\r\n
            buf_size += number_digit_count_u16(alias.resource_id) + 3;
            buf_size += number_digit_count_u16(alias.entry_index);
        }

        buf_size
    }

    pub fn to_ini_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity(self.calc_ini_byte_size());
        // note: extend_from_slice is benchmarked to be faster in 2022.03
        // [Global]\r\n
        vec.push('[' as u8);
        vec.extend_from_slice(PAK_INDEX_GLOBAL_TAG.as_bytes());
        vec.extend_from_slice(PAK_INDEX_TAG_END.as_bytes());
        // version=?\r\n
        vec.extend_from_slice(PAK_INDEX_GLOBAL_VERSION.as_bytes());
        vec.push('=' as u8);
        vec.extend_from_slice(self.header.read_version().to_string().as_bytes());
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // encoding=?\r\n
        vec.extend_from_slice(PAK_INDEX_GLOBAL_ENCODING.as_bytes());
        vec.push('=' as u8);
        vec.extend_from_slice(self.header.read_encoding().to_string().as_bytes());
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // \r\n
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // [Resources]\r\n
        vec.push('[' as u8);
        vec.extend_from_slice(PAK_INDEX_RES_TAG.as_bytes());
        vec.extend_from_slice(PAK_INDEX_TAG_END.as_bytes());
        // {resource_id}={file_name}\r\n
        for entry in self.entry_slice {
            vec.extend_from_slice(entry.resource_id.to_string().as_bytes());
            vec.push('=' as u8);
            vec.extend_from_slice(entry.file_name.as_bytes());
            vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        }

        if self.alias_slice.is_empty() {
            return vec;
        }
        // \r\n
        vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        // [Alias]\r\n
        vec.push('[' as u8);
        vec.extend_from_slice(PAK_INDEX_ALIAS_TAG.as_bytes());
        vec.extend_from_slice(PAK_INDEX_TAG_END.as_bytes());
        // {resource_id}={entry_index}\r\n
        for alias in self.alias_slice {
            vec.extend_from_slice(alias.resource_id.to_string().as_bytes());
            vec.push('=' as u8);
            vec.extend_from_slice(alias.entry_index.to_string().as_bytes());
            vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        }
        vec
    }
}

pub struct PakIndex {
    header: Box<dyn PakHeader>,
    entry_vec: Vec<PakIndexEntry>,
    alias_vec: Vec<PakAlias>,
}

impl PakIndex {
    #[inline]
    fn as_pak_index_ref(&self) -> PakIndexRef {
        PakIndexRef {
            header: self.header.as_ref(),
            entry_slice: &self.entry_vec,
            alias_slice: &self.alias_vec,
        }
    }

    #[inline]
    fn to_ini_bytes(&self) -> Vec<u8> {
        self.as_pak_index_ref().to_ini_bytes()
    }

    fn from_ini_buf(buf: &[u8]) -> Result<PakIndex, PakError> {
        // SAFETY: ini_core only uses as_bytes internally, the utf8 format has no effect
        let str: &str = unsafe { std::str::from_utf8_unchecked(buf) };
        let parser = ini_core::Parser::new(str);
        let mut status = PakIndexStatus::Init;
        let mut entry_vec: Vec<PakIndexEntry> = Vec::new();
        let mut alias_vec: Vec<PakAlias> = Vec::new();
        let mut version: u32 = 0;
        let mut encoding: u8 = 0;

        // parsing
        for item in parser {
            match item {
                Item::Error(err) => {
                    return Err(PakError::PakIndexParseError(String::from(err)));
                }
                Item::Section(section) => match section {
                    PAK_INDEX_GLOBAL_TAG => {
                        status = PakIndexStatus::Global;
                    }
                    PAK_INDEX_RES_TAG => {
                        status = PakIndexStatus::Resource;
                    }
                    PAK_INDEX_ALIAS_TAG => {
                        status = PakIndexStatus::Alias;
                    }
                    (other) => {
                        return Err(PakError::PakIndexUnknownTag(String::from(other)));
                    }
                },
                Item::Property(key, value) => match status {
                    PakIndexStatus::Init => {
                        return Err(PakError::PakIndexUnknownProperty(
                            status,
                            String::from(key),
                            String::from(value),
                        ));
                    }
                    PakIndexStatus::Global => match key {
                        PAK_INDEX_GLOBAL_VERSION => match u32::from_str(value) {
                            Ok(value) => {
                                if value == PAK_VERSION_V4 || value == PAK_VERSION_V5 {
                                    version = value;
                                } else {
                                    return Err(PakError::UnsupportedVersion(value));
                                }
                            }
                            Err(err) => {
                                return Err(PakError::PakIndexBadVersion(
                                    String::from(value),
                                    err,
                                ));
                            }
                        },
                        PAK_INDEX_GLOBAL_ENCODING => match u8::from_str(value) {
                            Ok(value) => {
                                encoding = value;
                            }
                            Err(err) => {
                                return Err(PakError::PakIndexBadVersion(
                                    String::from(value),
                                    err,
                                ));
                            }
                        }
                        (_) => {
                            return Err(PakError::PakIndexUnknownProperty(
                                status,
                                String::from(key),
                                String::from(value),
                            ));
                        }
                    },
                    PakIndexStatus::Resource => {
                        let resource_id: u16 = match u16::from_str(key) {
                            Ok(num) => num,
                            Err(err) => {
                                return Err(PakError::PakIndexBadResourceId(
                                    String::from(key), err));
                            }
                        };
                        let file_name: String = String::from(value);
                        entry_vec.push(PakIndexEntry { resource_id, file_name });
                    }
                    PakIndexStatus::Alias => {
                        if version == PAK_VERSION_V4 {
                            return Err(PakError::PakIndexAliasNotSupported(version));
                        }
                        let resource_id: u16 = match u16::from_str(key) {
                            Ok(num) => num,
                            Err(err) => {
                                return Err(PakError::PakIndexAliasBadResourceId(
                                    String::from(key),
                                    String::from(value), err));
                            }
                        };
                        let mut entry_index: u16 = match u16::from_str(value) {
                            Ok(num) => num,
                            Err(err) => {
                                return Err(PakError::PakIndexAliasBadEntryIndex(
                                    String::from(key),
                                    String::from(value), err));
                            }
                        };
                        alias_vec.push(PakAlias { resource_id, entry_index });
                    }
                }
                Item::Action(action) => {
                    return Err(PakError::PakIndexUnknownAction(
                        status, String::from(action)));
                }
                // ignore this?
                Item::Comment(_) => {}
                Item::Blank => {}
            }
        }

        let mut header = if version == PAK_VERSION_V5 {
            Box::new(PakHeaderV5::new())
        } else {
            // must be 4 here
            Box::new(PakHeaderV4::new())
        };
        header.write_version(version);
        Ok(PakIndex {
            header,
            entry_vec,
            alias_vec,
        })
    }
}
