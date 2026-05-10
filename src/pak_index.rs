use std::str::FromStr;

use ini_core::Item;

use crate::pak_def::{PakAlias, PakBase};
use crate::pak_error::PakError;
use crate::pak_format::PakFormat;
use crate::pak_header::{PAK_VERSION_V4, PAK_VERSION_V5, PakHeader, PakHeaderV4, PakHeaderV5};

pub enum PakIndexCompression {
    Raw,
    BrotliCompressed,
}

pub const PAK_INDEX_BROTLI_COMPRESSED: &str = ":::BrotliCompressed";
pub const PAK_INDEX_RAW: &str = ":::Raw";

impl PakIndexCompression {
    fn to_suffix(&self) -> &str {
        match self {
            PakIndexCompression::Raw => PAK_INDEX_RAW,
            PakIndexCompression::BrotliCompressed => PAK_INDEX_BROTLI_COMPRESSED
        }
    }

    fn strip_suffix(file_name: &mut String) {
        if file_name.ends_with(PAK_INDEX_RAW) {
            file_name.truncate(
                file_name.len() - PAK_INDEX_RAW.len())
        } else if file_name.ends_with(PAK_INDEX_BROTLI_COMPRESSED) {
            file_name.truncate(
                file_name.len() - PAK_INDEX_BROTLI_COMPRESSED.len())
        }
    }

    fn of_file_name(file_name: &str) -> PakIndexCompression {
        if file_name.ends_with(PAK_INDEX_BROTLI_COMPRESSED) {
            PakIndexCompression::BrotliCompressed
        } else {
            PakIndexCompression::Raw
        }
    }
}

pub struct PakIndexEntry {
    pub resource_id: u32,
    pub file_name: String,
    pub compression: PakIndexCompression
}

pub struct PakIndexRef<'a, T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> {
    pub header: &'a dyn PakHeader,
    pub entry_slice: &'a [PakIndexEntry],
    pub alias_slice: &'a [PakAlias<T>],
    pub format: PakFormat,
}

pub const PAK_INDEX_GLOBAL_TAG: &str = "Global";
pub const PAK_INDEX_RES_TAG: &str = "Resources";
pub const PAK_INDEX_ALIAS_TAG: &str = "Alias";
pub const PAK_INDEX_GLOBAL_VERSION: &str = "version";
pub const PAK_INDEX_GLOBAL_ENCODING: &str = "encoding";
pub const PAK_INDEX_GLOBAL_FORMAT: &str = "format";
pub const PAK_INDEX_FORMAT_EDGE_V5: &str = "edge-v5";
pub const PAK_INDEX_TAG_END: &str = "]\r\n";
pub const PAK_INDEX_CRLF: &str = "\r\n";

// naive but much benchmarked to be faster in 2022.09
// modified from https://stackoverflow.com/a/1489873
#[inline]
fn number_digit_count_u32(x: u32) -> usize {
    if x >= 10_000 {
        if x >= 10_000_000 {
            if x >= 100_000_000 {
                if x >= 1_000_000_000 {
                    10
                } else {
                    9
                }
            } else {
                8
            }
        } else if x >= 100_000 {
            if x >= 1_000_000 {
                7
            } else {
                6
            }
        } else {
            5
        }
    } else if x >= 100 {
        if x >= 1_000 {
            4
        } else {
            3
        }
    } else if x >= 10 {
        2
    } else {
        1
    }
}

#[inline]
fn number_digit_count_u16(x: u16) -> usize {
    if x >= 1000u16 {
        if x >= 10000u16 {
            return 5;
        }
        return 4;
    }
    if x >= 10u16 {
        if x >= 100u16 {
            return 3;
        }
        return 2;
    }
    return 1;
}

pub trait NumDigits {
    fn num_digits(&self) -> usize;
}

#[inline]
fn convert_u32<T, F>(value: u32, err: F) -> Result<T, PakError>
where
    T: TryFrom<u32>,
    F: FnOnce(u32) -> PakError,
{
    value.try_into().map_err(|_| err(value))
}

#[inline]
fn checked_count_u32<F>(value: usize, err: F) -> Result<u32, PakError>
where
    F: FnOnce(usize) -> PakError,
{
    value.try_into().map_err(|_| err(value))
}

impl NumDigits for u16 {
    #[inline]
    fn num_digits(&self) -> usize {
        number_digit_count_u16(self.clone())
    }
}

impl NumDigits for u32 {
    #[inline]
    fn num_digits(&self) -> usize {
        number_digit_count_u32(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PakIndexStatus {
    Init,
    Global,
    Resource,
    Alias,
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakIndexRef<'_, T> {
    fn calc_ini_byte_size(&self) -> usize {
        // 12: []\r\n * 2 + \r\n\r\n
        let mut buf_size: usize =
            PAK_INDEX_GLOBAL_TAG.len() + PAK_INDEX_RES_TAG.len() + 12;
        // 4: = \r\n + version number
        buf_size += PAK_INDEX_GLOBAL_VERSION.len() + 4;
        // 4: = \r\n + encoding number
        buf_size += PAK_INDEX_GLOBAL_ENCODING.len() + 4;
        if self.format == PakFormat::V5Edge {
            // 3: =\r\n
            buf_size += PAK_INDEX_GLOBAL_FORMAT.len() + PAK_INDEX_FORMAT_EDGE_V5.len() + 3;
        }
        if !self.alias_slice.is_empty() {
            // 8: \r\n\r\n + []\r\n
            buf_size += PAK_INDEX_ALIAS_TAG.len() + 8;
        }
        for entry in self.entry_slice {
            // 3: =\r\n
            buf_size += entry.resource_id.num_digits() + 3;
            buf_size += entry.file_name.len();
            if matches!(entry.compression, PakIndexCompression::BrotliCompressed) {
                buf_size += PAK_INDEX_BROTLI_COMPRESSED.len();
            }
        }
        for alias in self.alias_slice {
            // 3: =\r\n
            let resource_id = alias.read_resource_id_raw();
            buf_size += resource_id.num_digits() + 3;
            let entry_index = alias.read_entry_index_raw();
            buf_size += entry_index.num_digits();
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
        if self.format == PakFormat::V5Edge {
            // format=edge-v5\r\n
            vec.extend_from_slice(PAK_INDEX_GLOBAL_FORMAT.as_bytes());
            vec.push('=' as u8);
            vec.extend_from_slice(PAK_INDEX_FORMAT_EDGE_V5.as_bytes());
            vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        }
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
            if matches!(entry.compression, PakIndexCompression::BrotliCompressed) {
                vec.extend_from_slice(entry.compression.to_suffix().as_bytes());
            }
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
            vec.extend_from_slice(alias.read_resource_id().to_string().as_bytes());
            vec.push('=' as u8);
            vec.extend_from_slice(alias.read_entry_index().to_string().as_bytes());
            vec.extend_from_slice(PAK_INDEX_CRLF.as_bytes());
        }
        vec
    }
}

pub struct PakIndex<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> {
    pub header: Box<dyn PakHeader>,
    pub entry_vec: Vec<PakIndexEntry>,
    pub alias_vec: Vec<PakAlias<T>>,
    pub format: PakFormat,
}

impl <T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> PakIndex<T> {
    #[inline]
    #[allow(dead_code)]
    pub fn as_pak_index_ref(&self) -> PakIndexRef<T> {
        PakIndexRef {
            header: self.header.as_ref(),
            entry_slice: &self.entry_vec,
            alias_slice: &self.alias_vec,
            format: self.format,
        }
    }

    pub fn from_ini_buf(buf: &[u8]) -> Result<Self, PakError> {
        // SAFETY: ini_core only uses as_bytes internally, the utf8 format has no effect
        let str: &str = unsafe { std::str::from_utf8_unchecked(buf) };
        let parser = ini_core::Parser::new(str);
        let mut status = PakIndexStatus::Init;
        let mut entry_vec: Vec<PakIndexEntry> = Vec::new();
        let mut alias_vec: Vec<PakAlias<T>> = Vec::new();
        let mut version: u32 = 0;
        let mut encoding: u8 = 0;
        let mut format = PakFormat::V5Chromium;

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
                    other => {
                        return Err(PakError::PakIndexUnknownTag(String::from(other)));
                    }
                },
                Item::Property(key, value) => {
                    let value = match value {
                        Some(value) => value,
                        None => {
                            return Err(PakError::PakIndexUnknownAction(
                                status,
                                String::from(key),
                            ));
                        }
                    };
                    match status {
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
                                    return Err(PakError::PakIndexBadEncoding(
                                        String::from(value),
                                        err,
                                    ));
                                }
                            },
                            PAK_INDEX_GLOBAL_FORMAT => {
                                if value == PAK_INDEX_FORMAT_EDGE_V5 {
                                    format = PakFormat::V5Edge;
                                } else {
                                    return Err(PakError::PakIndexUnknownFormat(
                                        String::from(value),
                                    ));
                                }
                            }
                            _ => {
                                return Err(PakError::PakIndexUnknownProperty(
                                    status,
                                    String::from(key),
                                    String::from(value),
                                ));
                            }
                        },
                        PakIndexStatus::Resource => {
                            let resource_id = match u32::from_str(key) {
                                Ok(num) => num,
                                Err(err) => {
                                    return Err(PakError::PakIndexBadResourceId(
                                        String::from(key),
                                        err,
                                    ));
                                }
                            };
                            let _: T = convert_u32(
                                resource_id,
                                PakError::PakResourceIdOutOfRange,
                            )?;
                            let mut file_name: String = String::from(value);
                            let compression = PakIndexCompression::of_file_name(&file_name);
                            PakIndexCompression::strip_suffix(&mut file_name);
                            entry_vec.push(PakIndexEntry {
                                resource_id,
                                file_name,
                                compression,
                            });
                        }
                        PakIndexStatus::Alias => {
                            if version == 0 {
                                return Err(PakError::PakIndexMissingVersion);
                            }
                            if version == PAK_VERSION_V4 {
                                return Err(PakError::PakIndexAliasNotSupported(version));
                            }
                            let resource_id = match u32::from_str(key) {
                                Ok(num) => num,
                                Err(err) => {
                                    return Err(PakError::PakIndexAliasBadResourceId(
                                        String::from(key),
                                        String::from(value),
                                        err,
                                    ));
                                }
                            };
                            let entry_index = match u32::from_str(value) {
                                Ok(num) => num,
                                Err(err) => {
                                    return Err(PakError::PakIndexAliasBadEntryIndex(
                                        String::from(key),
                                        String::from(value),
                                        err,
                                    ));
                                }
                            };
                            let alias_resource_id = convert_u32(
                                resource_id,
                                PakError::PakAliasResourceIdOutOfRange,
                            )?;
                            let alias_entry_index = convert_u32(
                                entry_index,
                                PakError::PakAliasEntryIndexOutOfRange,
                            )?;
                            let mut alias = PakAlias::new();
                            alias.write_resource_id(alias_resource_id);
                            alias.write_entry_index(alias_entry_index);
                            alias_vec.push(alias);
                        }
                    }
                }
                // ignore this?
                Item::SectionEnd => {}
                Item::Comment(_) => {}
                Item::Blank => {}
            }
        }

        let mut header: Box<dyn PakHeader> = match version {
            PAK_VERSION_V5 => Box::new(PakHeaderV5::<T>::new()),
            PAK_VERSION_V4 => Box::new(PakHeaderV4::new()),
            _ => return Err(PakError::PakIndexMissingVersion),
        };
        if format == PakFormat::V5Edge && version != PAK_VERSION_V5 {
            return Err(PakError::PakIndexFormatVersionMismatch(
                String::from(PAK_INDEX_FORMAT_EDGE_V5),
                version,
            ));
        }
        if version == PAK_VERSION_V4 {
            format = PakFormat::V4;
        }
        for alias in &alias_vec {
            let entry_index = alias.read_entry_index();
            if entry_index as usize >= entry_vec.len() {
                return Err(PakError::PakAliasEntryIndexOutOfBounds(
                    alias.read_resource_id(),
                    entry_index,
                ));
            }
        }
        entry_vec.shrink_to_fit();
        header.write_encoding(encoding);
        let resource_count = checked_count_u32(
            entry_vec.len(),
            PakError::PakResourceCountOutOfRange,
        )?;
        header.write_resource_count(resource_count)?;
        if alias_vec.len() > 0 {
            let alias_count = checked_count_u32(
                alias_vec.len(),
                PakError::PakAliasCountOutOfRange,
            )?;
            header.write_alias_count(alias_count)?;
        }
        Ok(PakIndex {
            header,
            entry_vec,
            alias_vec,
            format,
        })
    }
}

pub fn pak_index_is_edge_v5(buf: &[u8]) -> Result<bool, PakError> {
    // SAFETY: ini_core only uses as_bytes internally, the utf8 format has no effect
    let str: &str = unsafe { std::str::from_utf8_unchecked(buf) };
    let parser = ini_core::Parser::new(str);
    let mut status = PakIndexStatus::Init;
    let mut version: u32 = 0;
    let mut is_edge_v5 = false;
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
                other => {
                    return Err(PakError::PakIndexUnknownTag(String::from(other)));
                }
            },
            Item::Property(key, value) => {
                if !matches!(status, PakIndexStatus::Global) {
                    continue;
                }
                let value = match value {
                    Some(value) => value,
                    None => {
                        return Err(PakError::PakIndexUnknownAction(
                            status,
                            String::from(key),
                        ));
                    }
                };
                match key {
                    PAK_INDEX_GLOBAL_VERSION => {
                        version = u32::from_str(value)
                            .map_err(|err| PakError::PakIndexBadVersion(
                                String::from(value),
                                err,
                            ))?;
                    }
                    PAK_INDEX_GLOBAL_FORMAT => {
                        if value == PAK_INDEX_FORMAT_EDGE_V5 {
                            is_edge_v5 = true;
                        } else {
                            return Err(PakError::PakIndexUnknownFormat(
                                String::from(value),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Item::SectionEnd => {}
            Item::Comment(_) => {}
            Item::Blank => {}
        }
    }
    if is_edge_v5 && version != PAK_VERSION_V5 {
        return Err(PakError::PakIndexFormatVersionMismatch(
            String::from(PAK_INDEX_FORMAT_EDGE_V5),
            version,
        ));
    }
    Ok(is_edge_v5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_digit_count_u16_test() {
        for i in 0..u16::MAX {
            let digits = number_digit_count_u16(i);
            assert_eq!(i.to_string().len(), digits);
        }
    }

    #[test]
    fn from_ini_rejects_out_of_range_resource_id_for_u16() {
        let buf = b"[Global]\nversion=5\nencoding=0\n\n[Resources]\n65536=65536.txt\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakResourceIdOutOfRange(65536)) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_rejects_out_of_range_alias_values_for_u16() {
        let buf = b"[Global]\nversion=5\nencoding=0\n\n[Resources]\n1=1.txt\n\n[Alias]\n65536=0\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakAliasResourceIdOutOfRange(65536)) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }

        let buf = b"[Global]\nversion=5\nencoding=0\n\n[Resources]\n1=1.txt\n\n[Alias]\n2=65536\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakAliasEntryIndexOutOfRange(65536)) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_rejects_out_of_range_resource_count_for_u16() {
        let mut ini = String::from("[Global]\nversion=5\nencoding=0\n\n[Resources]\n");
        for i in 0..=u16::MAX as u32 {
            ini.push_str(i.to_string().as_str());
            ini.push('=');
            ini.push_str(i.to_string().as_str());
            ini.push_str(".txt\n");
        }
        match PakIndex::<u16>::from_ini_buf(ini.as_bytes()) {
            Err(PakError::PakResourceCountOutOfRange(65536)) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_requires_version_before_alias_section() {
        let buf = b"[Alias]\n2=0\n\n[Global]\nversion=5\nencoding=0\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakIndexMissingVersion) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_requires_version_before_building_header() {
        let buf = b"[Global]\nencoding=0\n\n[Resources]\n1=1.txt\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakIndexMissingVersion) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_rejects_property_without_value_as_unknown_action() {
        let buf = b"[Global]\nversion=5\nunknown\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakIndexUnknownAction(PakIndexStatus::Global, action)) => {
                assert_eq!("unknown", action);
            }
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_rejects_alias_index_past_resource_count() {
        let buf = b"[Global]\nversion=5\nencoding=0\n\n[Resources]\n1=1.txt\n\n[Alias]\n2=1\n";
        match PakIndex::<u16>::from_ini_buf(buf) {
            Err(PakError::PakAliasEntryIndexOutOfBounds(2, 1)) => {}
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_accepts_edge_v5_format_marker_for_u32() {
        let buf = b"[Global]\nversion=5\nencoding=0\nformat=edge-v5\n\n[Resources]\n1=1.txt\n";
        let index = PakIndex::<u32>::from_ini_buf(buf).unwrap();
        assert_eq!(index.format, PakFormat::V5Edge);
    }

    #[test]
    fn from_ini_rejects_unknown_format_marker() {
        let buf = b"[Global]\nversion=5\nencoding=0\nformat=edge\n\n[Resources]\n1=1.txt\n";
        match PakIndex::<u32>::from_ini_buf(buf) {
            Err(PakError::PakIndexUnknownFormat(format)) => {
                assert_eq!(format, "edge");
            }
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn from_ini_rejects_edge_v5_format_marker_for_v4() {
        let buf = b"[Global]\nversion=4\nencoding=0\nformat=edge-v5\n\n[Resources]\n1=1.txt\n";
        match PakIndex::<u32>::from_ini_buf(buf) {
            Err(PakError::PakIndexFormatVersionMismatch(format, 4)) => {
                assert_eq!(format, PAK_INDEX_FORMAT_EDGE_V5);
            }
            other => panic!("unexpected result: {:?}", other.err()),
        }
    }

    #[test]
    fn pak_index_is_edge_v5_reads_format_marker_without_resource_validation() {
        let buf = b"[Global]\nversion=5\nencoding=0\nformat=edge-v5\n\n[Resources]\n65536=65536.txt\n";
        assert!(pak_index_is_edge_v5(buf).unwrap());
    }

}
