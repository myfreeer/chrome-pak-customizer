use crate::pak_def::{checked_add_usize, pak_parse_alias, PakBase, PakBaseOffset, PakEntry};
use crate::pak_error::PakError;
use crate::pak_header::{pak_get_version, PakHeader, PakHeaderV5, PAK_VERSION_V4, PAK_VERSION_V5};
use crate::pak_index::NumDigits;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PakFormat {
    V4,
    V5Chromium,
    V5Edge,
}

fn validate_v5<T>(pak_buf: &[u8]) -> Result<(), PakError>
where
    T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static,
{
    let header = PakHeaderV5::<T>::from_buf(pak_buf)?;
    let resource_count = header.read_resource_count();
    if resource_count == 0 {
        return Err(PakError::PakZeroResourceCount);
    }
    let resource_base_offset = checked_add_usize(
        checked_add_usize(header.size(), header.resource_size()?, "resource base offset")?,
        header.alias_size()?,
        "resource base offset",
    )?;
    if resource_base_offset > pak_buf.len() {
        return Err(PakError::PakPackResourceOffsetOverflow(
            0,
            resource_base_offset,
        ));
    }

    let mut entry_offset = header.size();
    let mut last_offset = None;
    for i in 0..=resource_count {
        let entry = PakEntry::<T>::from_buf_offset(pak_buf, entry_offset)?;
        let offset = entry.read_offset() as usize;
        if i == 0 && offset != resource_base_offset {
            return Err(PakError::PakPackResourceInvalidOffset(
                offset as u32,
                resource_base_offset as u32,
            ));
        }
        if let Some(last_offset) = last_offset {
            if last_offset > offset {
                return Err(PakError::PakPackResourceInvalidOffset(
                    last_offset as u32,
                    offset as u32,
                ));
            }
        }
        if offset > pak_buf.len() {
            return Err(PakError::PakPackResourceOffsetOverflow(
                entry.read_resource_id(),
                offset,
            ));
        }
        last_offset = Some(offset);
        entry_offset = checked_add_usize(
            entry_offset,
            std::mem::size_of::<PakEntry<T>>(),
            "resource table offset",
        )?;
    }

    let alias_slice = pak_parse_alias::<T>(header, pak_buf)?;
    for alias in alias_slice {
        let entry_index = alias.read_entry_index();
        if entry_index >= resource_count {
            return Err(PakError::PakAliasEntryIndexOutOfBounds(
                alias.read_resource_id(),
                entry_index,
            ));
        }
    }
    Ok(())
}

pub fn pak_format_from_buf(
    pak_buf: &[u8],
    force_edge_v5: bool,
) -> Result<PakFormat, PakError> {
    match pak_get_version(pak_buf)? {
        PAK_VERSION_V4 => Ok(PakFormat::V4),
        PAK_VERSION_V5 => {
            if force_edge_v5 {
                return Ok(PakFormat::V5Edge);
            }

            let chromium_result = validate_v5::<u16>(pak_buf);
            let edge_result = validate_v5::<u32>(pak_buf);
            match (chromium_result, edge_result) {
                (Ok(_), Err(_)) => Ok(PakFormat::V5Chromium),
                (Err(_), Ok(_)) => Ok(PakFormat::V5Edge),
                (Ok(_), Ok(_)) => Err(PakError::PakFormatAmbiguousV5),
                (Err(err), Err(_)) => Err(err),
            }
        }
        version => Err(PakError::UnsupportedVersion(version)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chromium_v5_pak() -> Vec<u8> {
        vec![
            5, 0, 0, 0, // version
            0, // encoding
            0, 0, 0, // padding
            1, 0, // resource count
            0, 0, // alias count
            1, 0, 24, 0, 0, 0, // resource id 1, offset 24
            0, 0, 29, 0, 0, 0, // final entry, end offset 29
            b'h', b'e', b'l', b'l', b'o',
        ]
    }

    fn edge_v5_pak() -> Vec<u8> {
        vec![
            5, 0, 0, 0, // version
            0, // encoding
            0, 0, 0, // padding
            1, 0, 0, 0, // resource count
            0, 0, 0, 0, // alias count
            1, 0, 0, 0, 32, 0, 0, 0, // resource id 1, offset 32
            0, 0, 0, 0, 37, 0, 0, 0, // final entry, end offset 37
            b'h', b'e', b'l', b'l', b'o',
        ]
    }

    #[test]
    fn auto_detects_chromium_v5() {
        assert_eq!(
            pak_format_from_buf(&chromium_v5_pak(), false).unwrap(),
            PakFormat::V5Chromium,
        );
    }

    #[test]
    fn auto_detects_edge_v5() {
        assert_eq!(
            pak_format_from_buf(&edge_v5_pak(), false).unwrap(),
            PakFormat::V5Edge,
        );
    }

    #[test]
    fn forced_edge_v5_skips_auto_detection() {
        assert_eq!(
            pak_format_from_buf(&chromium_v5_pak(), true).unwrap(),
            PakFormat::V5Edge,
        );
    }
}
