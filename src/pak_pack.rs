// pub fn pak_pack

use std::fs::{File, read};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::pak_def::{PakAlias, PakBase, PakEntry, checked_add_usize};
use crate::pak_error::PakError;
use crate::pak_error::PakError::{
    PakPackResourceOffsetOverflow,
    PakPackWriteFileError,
    PakReadIndexFileFail
};
use crate::pak_file_io::{PakFileContent, pak_read_files};
use crate::pak_index::{pak_index_is_edge_v5, NumDigits};
use crate::PakIndex;

struct PakPackParts<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static> {
    pak_index: PakIndex<T>,
    pak_files: Vec<PakFileContent>,
    resource_entries: Vec<PakEntry<T>>,
    alias_size: usize,
    total_size: usize,
}

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
    if edge_v5 || pak_index_is_edge_v5(&index_file)? {
        pak_pack_index_path_impl::<u32>(&index_file, index_dir, output_path)
    } else {
        pak_pack_index_path_impl::<u16>(&index_file, index_dir, output_path)
    }
}

pub fn pak_pack_index_vec<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    pak_index_buf: &[u8],
    index_dir: &Path
) -> Result<Vec<u8>, PakError> {
    let parts = pak_pack_parts::<T>(pak_index_buf, index_dir)?;
    let mut vec = Vec::with_capacity(parts.total_size);
    pak_pack_write(
        &parts.pak_index,
        &parts.pak_files,
        &parts.resource_entries,
        parts.alias_size,
        &mut vec,
    ).expect("Vec<u8> writes are infallible");
    Ok(vec)
}

fn pak_pack_parts<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    pak_index_buf: &[u8],
    index_dir: &Path
) -> Result<PakPackParts<T>, PakError> {
    let pak_index = PakIndex::<T>::from_ini_buf(pak_index_buf)?;
    let pak_files = pak_read_files(index_dir, &pak_index.entry_vec)?;
    let pak_header = pak_index.header.as_ref();
    let header_size = pak_header.size();
    let resource_size = pak_header.resource_size()?;
    let alias_size = pak_header.alias_size()?;
    let resource_base_offset = checked_add_usize(
        checked_add_usize(header_size, resource_size, "resource base offset")?,
        alias_size,
        "resource base offset",
    )?;
    let mut resource_offset = resource_base_offset;
    let mut resource_entries = Vec::with_capacity(pak_files.len() + 1);
    for file in &pak_files {
        let resource_id = file.resource_id
            .try_into()
            .map_err(|_| PakError::PakResourceIdOutOfRange(file.resource_id))?;
        if resource_offset > u32::MAX as usize {
            return Err(PakPackResourceOffsetOverflow(
                file.resource_id, resource_offset));
        }
        let mut resource_entry = PakEntry::new();
        resource_entry.write_resource_id(resource_id);
        resource_entry.write_offset(resource_offset as u32);
        resource_entries.push(resource_entry);
        resource_offset = checked_add_usize(
            resource_offset,
            file.content.len(),
            "resource offset",
        )?;
    }
    if resource_offset > u32::MAX as usize {
        return Err(PakPackResourceOffsetOverflow(0, resource_offset));
    }
    let mut resource_entry = PakEntry::new();
    resource_entry.write_offset(resource_offset as u32);
    resource_entries.push(resource_entry);

    Ok(PakPackParts {
        pak_index,
        pak_files,
        resource_entries,
        alias_size,
        total_size: resource_offset,
    })
}

fn pak_pack_index_path_impl<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    pak_index_buf: &[u8],
    index_dir: &Path,
    output_path: String,
) -> Result<(), PakError> {
    let parts = pak_pack_parts::<T>(pak_index_buf, index_dir)?;
    let file = File::create(Path::new(&output_path))
        .map_err(|err| PakPackWriteFileError(output_path.clone(), err))?;
    let mut writer = BufWriter::new(file);
    pak_pack_write(
        &parts.pak_index,
        &parts.pak_files,
        &parts.resource_entries,
        parts.alias_size,
        &mut writer,
    ).map_err(|err| PakPackWriteFileError(output_path.clone(), err))?;
    writer.flush()
        .map_err(|err| PakPackWriteFileError(output_path, err))
}

fn pak_pack_write<T, W>(
    pak_index: &PakIndex<T>,
    pak_files: &[PakFileContent],
    resource_entries: &[PakEntry<T>],
    alias_size: usize,
    writer: &mut W,
) -> std::io::Result<()>
where
    T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static,
    W: Write,
{
    writer.write_all(pak_index.header.as_bytes())?;
    for entry in resource_entries {
        writer.write_all(entry.as_bytes())?;
    }
    writer.write_all(PakAlias::serialize_slice(
        &pak_index.alias_vec,
        alias_size,
    ))?;
    for file in pak_files {
        writer.write_all(&file.content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn pak_pack_index_path_matches_vec_output() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "chrome-pak-customizer-pack-{0}-{1}",
            std::process::id(),
            unique,
        ));
        fs::create_dir_all(&dir).unwrap();

        let index_path = dir.join("pak_index.ini");
        let output_path = dir.join("out.pak");
        fs::write(dir.join("1.txt"), b"hello world").unwrap();
        fs::write(
            &index_path,
            b"[Global]\nversion=4\nencoding=0\n\n[Resources]\n1=1.txt\n",
        ).unwrap();

        let vec = pak_pack_index_vec::<u16>(
            fs::read(&index_path).unwrap().as_slice(),
            &dir,
        ).unwrap();
        pak_pack_index_path(
            index_path.to_string_lossy().into_owned(),
            output_path.to_string_lossy().into_owned(),
            false,
        ).unwrap();
        let file_bytes = fs::read(&output_path).unwrap();
        assert_eq!(file_bytes, vec);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pak_pack_index_path_uses_edge_v5_marker_without_flag() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "chrome-pak-customizer-pack-edge-{0}-{1}",
            std::process::id(),
            unique,
        ));
        fs::create_dir_all(&dir).unwrap();

        let index_path = dir.join("pak_index.ini");
        let output_path = dir.join("out.pak");
        fs::write(dir.join("1.txt"), b"hello").unwrap();
        fs::write(
            &index_path,
            b"[Global]\nversion=5\nencoding=0\nformat=edge-v5\n\n[Resources]\n1=1.txt\n",
        ).unwrap();

        pak_pack_index_path(
            index_path.to_string_lossy().into_owned(),
            output_path.to_string_lossy().into_owned(),
            false,
        ).unwrap();
        let pak = fs::read(&output_path).unwrap();
        assert_eq!(&pak[0..4], &[5, 0, 0, 0]);
        assert_eq!(&pak[8..12], &[1, 0, 0, 0]);
        assert_eq!(&pak[12..16], &[0, 0, 0, 0]);

        fs::remove_dir_all(&dir).unwrap();
    }

}
