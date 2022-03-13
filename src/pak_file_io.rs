use std::fs::read;
use std::path::Path;

use crate::pak_error::PakError;
use crate::pak_error::PakError::{PakPackReadResourceError, PakReadIndexFileFail};
use crate::pak_file::PakFile;
use crate::pak_file_type::{BROTLI_HEADER_SIZE, pak_get_file_type, PakFileCompression};
use crate::pak_index::PakIndexEntry;

pub fn pak_write_file(dir: &String, pak_file: &PakFile) -> Result<PakIndexEntry, PakError> {
    let file_type = pak_get_file_type(pak_file.buf);
    let mut buf = pak_file.buf;
    // strip chromium brotli header
    if matches!(file_type.compression, PakFileCompression::ChromiumBrotli) &&
        buf.len() > BROTLI_HEADER_SIZE {
        buf = &buf[BROTLI_HEADER_SIZE..];
    }
    let mut file_name = pak_file.id.to_string();
    if file_type.ext_name.len() > 0 {
        file_name.push_str(file_type.ext_name);
    }
    let mut target_file_path = dir.clone();
    target_file_path.push(std::path::MAIN_SEPARATOR);
    target_file_path.push_str(file_name.as_str());
    let path = Path::new(&target_file_path);
    let result = std::fs::write(path, buf);
    match result {
        Ok(_) => {
            Ok(PakIndexEntry { resource_id: pak_file.id, file_name })
        }
        Err(err) => {
            Err(PakError::PakWriteFileFail(target_file_path, err))
        }
    }
}

pub struct PakFileContent {
    pub resource_id: u16,
    pub content: Vec<u8>,
}

pub fn pak_read_files(dir: &Path, entries: &[PakIndexEntry])
                      -> Result<Vec<PakFileContent>, PakError> {
    let mut vec = Vec::with_capacity(entries.len());
    for entry in entries {
        let file_path_buf = dir.join(&entry.file_name);
        let file_path = file_path_buf.as_path();
        // TODO: compression
        match read(file_path) {
            Ok(content) => vec.push(PakFileContent {
                resource_id: entry.resource_id,
                content
            }),
            Err(err) => {
                return Err(PakPackReadResourceError(file_path_buf, err));
            }
        }
    }
    Ok(vec)
}
