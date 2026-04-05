use std::fs::read;
use std::path::Path;

use PakError::PakWriteFileFail;

use crate::pak_brotli::brotli_calculate_decompressed_size;
use crate::pak_error::PakError;
use crate::pak_error::PakError::{
    PakPackBrotliDecompressError,
    PakPackReadResourceError,
};
use crate::pak_file::PakFile;
use crate::pak_file_type::{
    BROTLI_CONST,
    BROTLI_HEADER_SIZE,
    pak_get_file_type,
    PakFileCompression,
};
#[cfg(debug_assertions)]
use crate::pak_file_type::pak_get_chromium_brotli_decompressed_size;
use crate::pak_index::PakIndexCompression;
use crate::pak_index::PakIndexEntry;

pub fn pak_write_file(dir: &String, pak_file: &PakFile)
                      -> Result<PakIndexEntry, PakError> {
    let file_type = pak_get_file_type(pak_file.buf);
    let mut buf = pak_file.buf;
    // strip chromium brotli header
    if matches!(file_type.compression, PakFileCompression::ChromiumBrotli) &&
        buf.len() > BROTLI_HEADER_SIZE {
        #[cfg(debug_assertions)]
        println!("br: id={0}, decompressed_size={1}",
                 pak_file.id,
                 pak_get_chromium_brotli_decompressed_size(&buf).unwrap());
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
    if let Err(err) = std::fs::write(path, buf) {
        Err(PakWriteFileFail(target_file_path, err))
    } else {
        Ok(PakIndexEntry {
            resource_id: pak_file.id,
            file_name,
            compression: file_type.compression.into(),
        })
    }
}

pub struct PakFileContent {
    pub resource_id: u32,
    pub content: Vec<u8>,
}

pub fn pak_read_files(dir: &Path, entries: &[PakIndexEntry])
                      -> Result<Vec<PakFileContent>, PakError> {
    let mut vec = Vec::with_capacity(entries.len());
    for entry in entries {
        let file_path_buf = dir.join(&entry.file_name);
        let file_path = file_path_buf.as_path();
        let file_content = read(file_path)
            .map_err(|err| PakPackReadResourceError(file_path_buf.clone(), err))?;
        let content = match entry.compression {
            PakIndexCompression::Raw => file_content,
            PakIndexCompression::BrotliCompressed => {
                let mut content = Vec::with_capacity(
                    file_content.len() + BROTLI_HEADER_SIZE);
                // BROTLI_CONST is prepended to brotli decompressed data in order to
                // easily check if a resource has been brotli compressed.
                content.extend_from_slice(BROTLI_CONST.as_slice());
                // The length of the uncompressed data as 8 bytes little-endian.
                let size = brotli_calculate_decompressed_size(&file_content)
                    .map_err(|err| PakPackBrotliDecompressError(
                        file_path_buf.clone(),
                        err,
                    ))?;
                let bytes = size.to_le_bytes();
                // The length of the uncompressed data is also appended to the start,
                // truncated to 6 bytes, little-endian. size_bytes is 8 bytes,
                // need to truncate further to 6.
                content.extend_from_slice(
                    &bytes[..(BROTLI_HEADER_SIZE - BROTLI_CONST.len())]);
                content.extend_from_slice(&file_content);
                content
            }
        };
        vec.push(PakFileContent {
            resource_id: entry.resource_id,
            content,
        });
    }
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn pak_read_files_returns_error_for_invalid_brotli_resource() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "chrome-pak-customizer-brotli-{0}-{1}",
            std::process::id(),
            unique,
        ));
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("1.br");
        fs::write(&file_path, b"not a valid brotli payload").unwrap();

        let entries = [PakIndexEntry {
            resource_id: 1,
            file_name: String::from("1.br"),
            compression: PakIndexCompression::BrotliCompressed,
        }];

        let result = pak_read_files(&dir, &entries);
        assert!(matches!(
            result,
            Err(PakError::PakPackBrotliDecompressError(path, _))
                if path == file_path
        ));

        fs::remove_dir_all(&dir).unwrap();
    }
}
