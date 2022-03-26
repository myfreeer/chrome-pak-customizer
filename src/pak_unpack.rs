use std::fs;
use std::path::Path;

use crate::pak_def::pak_parse_alias;
use crate::pak_error::PakError;
use crate::pak_error::PakError::{
    PakUnpackCanNotCreateOutputPath,
    PakUnpackOutputPathNotDir,
    PakUnpackPakReadError,
    PakUnpackPathNotExists,
    PakWriteIndexFileFail
};
use crate::pak_file::pak_parse_files;
use crate::pak_file_io::pak_write_file;
use crate::pak_header::pak_read_header;
use crate::pak_index::PakIndexRef;

pub const PAK_INDEX_INI: &str = "pak_index.ini";

pub fn pak_unpack_path(pak_path_str: String, output_path: String) -> Result<(), PakError> {
    let pak_path = Path::new(&pak_path_str);
    if !pak_path.exists() {
        return Err(PakUnpackPathNotExists(pak_path_str));
    }
    let vec = fs::read(pak_path)
        .map_err(|err|  PakUnpackPakReadError(pak_path_str, err))?;

    pak_unpack_buf(&vec, output_path)
}

pub fn pak_unpack_buf(pak_buf: &[u8], output_path_str: String) -> Result<(), PakError> {
    let output_path = Path::new(&output_path_str);
    match fs::metadata(output_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(PakUnpackOutputPathNotDir(output_path_str));
            }
        }
        Err(_) => {
            match fs::create_dir_all(output_path) {
                Ok(_) => {}
                Err(err) => {
                    return Err(PakUnpackCanNotCreateOutputPath(
                        output_path_str, err));
                }
            }
        }
    }

    let header = pak_read_header(pak_buf)?;

    let files = pak_parse_files(header, pak_buf)?;
    let mut entry_vec = Vec::with_capacity(files.len());
    for x in files {
        let entry = pak_write_file(&output_path_str, &x)?;
        entry_vec.push(entry);
    }

    let alias_slice = pak_parse_alias(header, pak_buf)?;

    let index = PakIndexRef {
        header,
        entry_slice: &entry_vec,
        alias_slice,
    };

    let mut index_path_str = output_path_str.clone();
    index_path_str.push(std::path::MAIN_SEPARATOR);
    index_path_str.push_str(PAK_INDEX_INI);

    fs::write(Path::new(&index_path_str), index.to_ini_bytes())
        .map_err(|err| PakWriteIndexFileFail(index_path_str, err))
}