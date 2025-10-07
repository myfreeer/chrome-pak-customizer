// pub fn pak_pack

use std::fs::{read, write};
use std::path::Path;

use crate::pak_def::{PakAlias, PakBase, PakEntry};
use crate::pak_error::PakError;
use crate::pak_error::PakError::{
    PakPackResourceOffsetOverflow,
    PakPackWriteFileError,
    PakReadIndexFileFail
};
use crate::pak_file_io::pak_read_files;
use crate::pak_index::NumDigits;
use crate::PakIndex;

pub fn pak_pack_index_path(
    index_path_str: String,
    output_path: String,
    edge_v5: bool,
) -> Result<(), PakError> {
    let index_path = Path::new(&index_path_str);
    let index_dir = index_path.parent().unwrap_or(Path::new(""));
    let index_file = match read(index_path) {
        Ok(vec) => vec,
        Err(err) => {
            return Err(PakReadIndexFileFail(index_path_str, err));
        }
    };
    let packed = if edge_v5 {
        pak_pack_index_vec::<u32>(&index_file, index_dir)
    } else {
        pak_pack_index_vec::<u16>(&index_file, index_dir)
    }?;
    write(Path::new(&output_path), packed)
        .map_err(|err| PakPackWriteFileError(output_path, err))
}

pub fn pak_pack_index_vec<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    pak_index_buf: &[u8],
    index_dir: &Path
) -> Result<Vec<u8>, PakError> {
    let pak_index = PakIndex::<T>::from_ini_buf(pak_index_buf)?;
    let pak_files = pak_read_files(index_dir, &pak_index.entry_vec)?;
    // header
    let pak_header = pak_index.header.as_ref();
    let header_size = pak_header.size();
    let resource_size = pak_header.resource_size();
    let alias_size = pak_header.alias_size();
    let mut vec_size = header_size + resource_size + alias_size;
    for file in &pak_files {
        vec_size += file.content.len();
    }
    let mut vec = Vec::with_capacity(vec_size);
    vec.extend_from_slice(pak_header.as_bytes());

    // resource entry
    let resource_base_offset = header_size + resource_size + alias_size;
    let mut resource_offset = resource_base_offset;
    let mut resource_entry: PakEntry<T> = PakEntry {
        resource_id: 0u32.try_into().unwrap_or_default(),
        offset: 0u32.try_into().unwrap_or_default(),
    };
    for file in &pak_files {
        resource_entry.resource_id = file.resource_id.try_into().unwrap_or_default();
        if resource_offset > u32::MAX as usize {
            return Err(PakPackResourceOffsetOverflow(
                file.resource_id, resource_offset));
        }
        resource_entry.offset = resource_offset as u32;
        resource_offset += file.content.len();
        vec.extend_from_slice(resource_entry.as_bytes());
    }
    resource_entry.resource_id = 0u32.try_into().unwrap_or_default();
    if resource_offset > u32::MAX as usize {
        return Err(PakPackResourceOffsetOverflow(0, resource_offset));
    }
    resource_entry.offset = resource_offset as u32;
    vec.extend_from_slice(resource_entry.as_bytes());

    // alias
    let alias_slice = &pak_index.alias_vec;
    vec.extend_from_slice(PakAlias::serialize_slice(alias_slice, alias_size));

    for file in &pak_files {
        vec.extend_from_slice(&file.content);
    }

    Ok(vec)
}