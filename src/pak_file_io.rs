use std::path::Path;

use crate::pak_error::PakError;
use crate::pak_file::PakFile;
use crate::pak_file_type::pak_get_file_type;

pub fn pak_write_file(dir: &String, pak_file: &PakFile) -> Result<String, PakError> {
    let file_type = pak_get_file_type(pak_file.buf);
    let mut file_name = pak_file.id.to_string();
    if file_type.ext_name.len() > 0 {
        file_name.push_str(file_type.ext_name);
    }
    let mut target_file_path = String::from(dir);
    target_file_path.push(std::path::MAIN_SEPARATOR);
    target_file_path.push_str(file_name.as_str());
    let path = Path::new(&target_file_path);
    let result = std::fs::write(path, pak_file.buf);
    match result {
        Ok(_) => {Ok(file_name) }
        Err(err) => {
            Err(PakError::PakWriteFileFail(err))
        }
    }
}
