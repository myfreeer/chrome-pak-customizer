use std::fs;
use std::path::Path;

use crate::pak_def::pak_parse_alias;
use crate::pak_error::PakError;
use crate::pak_error::PakError::{
    PakUnpackCanNotCreateOutputPath,
    PakUnpackOutputPathNotDir,
    PakUnpackPakMapReadError,
    PakUnpackPakReadError,
    PakUnpackPathNotExists,
    PakWriteIndexFileFail
};
use crate::pak_format::{pak_format_from_buf, PakFormat};
use crate::pak_file::pak_parse_files;
use crate::pak_file_io::pak_write_file;
use crate::pak_header::{pak_read_header, PakHeader};
use crate::pak_index::{NumDigits, PakIndexEntry, PakIndexRef};

pub const PAK_INDEX_INI: &str = "pak_index.ini";

pub fn pak_unpack_path(
    pak_path_str: String,
    output_path: String,
    edge_v5: bool,
    mmap: bool,
) -> Result<(), PakError> {
    let pak_path = Path::new(&pak_path_str);
    if !pak_path.exists() {
        return Err(PakUnpackPathNotExists(pak_path_str));
    }
    if mmap && !crate::pak_mmap::MMAP_AVAILABLE {
        eprintln!("Warning: memory-mapped IO is not supported on this target; using buffered IO");
    }
    if mmap && crate::pak_mmap::MMAP_AVAILABLE {
        let metadata = fs::metadata(pak_path)
            .map_err(|err| PakUnpackPakReadError(pak_path_str.clone(), err))?;
        if metadata.len() > 0 {
            let map = crate::pak_mmap::map_read_only(pak_path)
                .map_err(|err| PakUnpackPakMapReadError(pak_path_str, err))?;
            return pak_unpack_buf(&map, output_path, edge_v5);
        }
    }
    let vec = fs::read(pak_path)
        .map_err(|err|  PakUnpackPakReadError(pak_path_str, err))?;

    pak_unpack_buf(&vec, output_path, edge_v5)
}

pub fn pak_unpack_buf(pak_buf: &[u8], output_path_str: String, edge_v5: bool) -> Result<(), PakError> {
    let output_path = Path::new(&output_path_str);
    match fs::metadata(output_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(PakUnpackOutputPathNotDir(output_path_str));
            }
        }
        Err(_) => {
            if let Err(err) = fs::create_dir_all(output_path) {
                return Err(PakUnpackCanNotCreateOutputPath(
                    output_path_str, err));
            }
        }
    }

    let format = pak_format_from_buf(pak_buf, edge_v5)?;
    let edge_v5 = format == PakFormat::V5Edge;
    let header = pak_read_header(pak_buf, edge_v5)?;
    let files = if edge_v5 {
        pak_parse_files::<u32>(header, pak_buf)
    } else {
        pak_parse_files::<u16>(header, pak_buf)
    }?;
    let mut entry_vec = Vec::with_capacity(files.len());
    for x in files {
        let entry = pak_write_file(&output_path_str, &x)?;
        entry_vec.push(entry);
    }

    if edge_v5 {
        pak_write_index::<u32>(header, pak_buf, entry_vec, format, &output_path_str)
    } else {
        pak_write_index::<u16>(header, pak_buf, entry_vec, format, &output_path_str)
    }
}

fn pak_write_index<T: Copy + Into<u32> + Default + TryFrom<u32> + NumDigits + 'static>(
    header: & dyn PakHeader,
    pak_buf: &[u8],
    entry_vec: Vec<PakIndexEntry>,
    format: PakFormat,
    output_path_str: &String
) -> Result<(), PakError> {

    let alias_slice = pak_parse_alias::<T>(header, pak_buf)?;

    let index = PakIndexRef {
        header,
        entry_slice: &entry_vec,
        alias_slice,
        format,
    };

    let mut index_path_str = output_path_str.clone();
    index_path_str.push(std::path::MAIN_SEPARATOR);
    index_path_str.push_str(PAK_INDEX_INI);

    fs::write(Path::new(&index_path_str), index.to_ini_bytes())
        .map_err(|err| PakWriteIndexFileFail(index_path_str, err))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "chrome-pak-customizer-{0}-{1}-{2}",
            name,
            std::process::id(),
            unique,
        ))
    }

    #[test]
    fn pak_unpack_path_with_mmap_matches_buffered_output() {
        let dir = unique_temp_dir("unpack-mmap");
        let buffered_dir = dir.join("buffered");
        let mmap_dir = dir.join("mmap");
        let pak_path = dir.join("test.pak");
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            &pak_path,
            [
                4, 0, 0, 0, // version
                1, 0, 0, 0, // resource count
                0, // encoding
                1, 0, 21, 0, 0, 0, // resource id 1, offset 21
                0, 0, 26, 0, 0, 0, // final entry, end offset 26
                b'h', b'e', b'l', b'l', b'o',
            ],
        ).unwrap();

        pak_unpack_path(
            pak_path.to_string_lossy().into_owned(),
            buffered_dir.to_string_lossy().into_owned(),
            false,
            false,
        ).unwrap();
        pak_unpack_path(
            pak_path.to_string_lossy().into_owned(),
            mmap_dir.to_string_lossy().into_owned(),
            false,
            true,
        ).unwrap();

        assert_eq!(fs::read(buffered_dir.join("1")).unwrap(), b"hello");
        assert_eq!(fs::read(mmap_dir.join("1")).unwrap(), b"hello");
        assert_eq!(
            fs::read(buffered_dir.join(PAK_INDEX_INI)).unwrap(),
            fs::read(mmap_dir.join(PAK_INDEX_INI)).unwrap(),
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pak_unpack_path_with_mmap_preserves_zero_length_parser_error() {
        let dir = unique_temp_dir("unpack-mmap-empty");
        let output_dir = dir.join("out");
        let pak_path = dir.join("empty.pak");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&pak_path, []).unwrap();

        let result = pak_unpack_path(
            pak_path.to_string_lossy().into_owned(),
            output_dir.to_string_lossy().into_owned(),
            false,
            true,
        );

        assert!(matches!(
            result,
            Err(PakError::VersionSizeNotEnough(0, 4))
        ));
        assert!(output_dir.is_dir());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn pak_unpack_path_with_mmap_returns_map_error_for_directory_input() {
        let dir = unique_temp_dir("unpack-mmap-map-error");
        let pak_path = dir.join("input-dir");
        let output_dir = dir.join("out");
        fs::create_dir_all(&pak_path).unwrap();

        let result = pak_unpack_path(
            pak_path.to_string_lossy().into_owned(),
            output_dir.to_string_lossy().into_owned(),
            false,
            true,
        );

        assert!(matches!(
            result,
            Err(PakError::PakUnpackPakMapReadError(_, _))
        ));
        assert!(!output_dir.exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pak_unpack_path_auto_writes_edge_v5_format_marker() {
        let dir = unique_temp_dir("unpack-edge-v5-auto");
        let out_dir = dir.join("out");
        let pak_path = dir.join("edge.pak");
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            &pak_path,
            [
                5, 0, 0, 0, // version
                0, // encoding
                0, 0, 0, // padding
                1, 0, 0, 0, // resource count
                0, 0, 0, 0, // alias count
                1, 0, 0, 0, 32, 0, 0, 0, // resource id 1, offset 32
                0, 0, 0, 0, 37, 0, 0, 0, // final entry, end offset 37
                b'h', b'e', b'l', b'l', b'o',
            ],
        ).unwrap();

        pak_unpack_path(
            pak_path.to_string_lossy().into_owned(),
            out_dir.to_string_lossy().into_owned(),
            false,
            false,
        ).unwrap();

        let index = fs::read_to_string(out_dir.join(PAK_INDEX_INI)).unwrap();
        assert!(index.contains("format=edge-v5"));
        assert_eq!(fs::read(out_dir.join("1")).unwrap(), b"hello");

        fs::remove_dir_all(&dir).unwrap();
    }
}
