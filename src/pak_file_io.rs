use std::path::Path;

use crate::pak_error::PakError;
use crate::pak_file::PakFile;
use crate::pak_file_type::pak_get_file_type;

pub fn pak_write_file(dir: &String, pak_file: &PakFile) -> Result<(), PakError> {
    let file_type = pak_get_file_type(pak_file.buf);
    let mut target_file_path = String::from(dir);
    target_file_path.push(std::path::MAIN_SEPARATOR);
    target_file_path.push_str(pak_file.id.to_string().as_str());
    if file_type.ext_name.len() > 0 {
        target_file_path.push_str(file_type.ext_name);
    }
    let path = Path::new(&target_file_path);
    let result = std::fs::write(path, pak_file.buf);
    match result {
        Ok(_) => {Ok(()) }
        Err(err) => {
            Err(PakError::PakWriteFileFail(err))
        }
    }
}
