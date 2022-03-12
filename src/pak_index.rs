use crate::pak_def::PakAlias;
use crate::pak_header::PakHeader;

pub struct PakIndexEntry {
    resource_id: u16,
    file_name: String,
}

pub struct PakIndex<'a> {
    header: &'a dyn PakHeader,
    entry_slice: &'a [PakIndexEntry],
    alias_slice: &'a [PakAlias]
}

static NUMBER_DECIMAL_U32: [u32; 10] = [
    10, 100, 1000, 10000, 100000, 1000000,
    10000000, 100000000, 1000000000, u32::MAX
];

fn number_digit_count_u32(num: u32) -> u32 {
    let mut count: u32 = 1;
    for x in NUMBER_DECIMAL_U32 {
        if num < x {
            return count;
        }
        count += 1;
    }
    count
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

// TODO: pak index <- ini file
// TODO: pak index <-> pak header + buf
impl PakIndex<'_> {
    fn calc_ini_byte_size(&self) -> usize {
        // 12: []\r\n * 2 + \r\n\r\n
        let mut buf_size: usize =
            PAK_INDEX_GLOBAL_TAG.len() + PAK_INDEX_RES_TAG.len() + 12;
        // 4: = \r\n + version number
        buf_size += PAK_INDEX_GLOBAL_VERSION.len() + 4;
        // 4: = \r\n + encoding number
        buf_size += PAK_INDEX_GLOBAL_ENCODING.len() + 4;
        if ! self.alias_slice.is_empty() {
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

    fn to_ini_bytes(&self) -> Vec<u8> {
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
