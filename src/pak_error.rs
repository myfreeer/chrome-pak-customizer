use std::io::Error;
use std::num::ParseIntError;
use std::path::PathBuf;

use crate::pak_index::PakIndexStatus;

#[derive(Debug)]
pub enum PakError {
    #[allow(dead_code)]
    Unknown,
    // actual, expected
    VersionSizeNotEnough(usize, usize),
    // version
    UnsupportedVersion(u32),
    // actual, expected
    VersionMisMatch(u32, u32),
    // actual, expected
    V4HeaderSizeNotEnough(usize, usize),
    // actual, expected
    V5HeaderSizeNotEnough(usize, usize),
    // buffer length, offset
    PakEntryOrAliasOffsetOverflow(usize, usize),
    // actual, expected
    PakEntryOrAliasSizeNotEnough(usize, usize),
    PakZeroResourceCount,
    PakWriteFileFail(String, Error),
    // buffer length, offset
    PakAliasOffsetOverflow(usize, usize),
    // actual, expected
    PakAliasSizeNotEnough(usize, usize),
    #[allow(dead_code)]
    PakNotChromiumBrotli,
    #[allow(dead_code)]
    PakChromiumBrotliSizeNotEnough(usize),
    PakIndexParseError(String),
    PakIndexUnknownTag(String),
    // status, key, value
    PakIndexUnknownProperty(PakIndexStatus, String, String),
    // version str, err
    PakIndexBadVersion(String, ParseIntError),
    PakIndexBadResourceId(String, ParseIntError),
    // version
    PakIndexAliasNotSupported(u32),
    // key, value, err
    PakIndexAliasBadResourceId(String, String, ParseIntError),
    // key, value, err
    PakIndexAliasBadEntryIndex(String, String, ParseIntError),
    PakIndexUnknownAction(PakIndexStatus, String),
    PakUnpackPathNotExists(String),
    PakUnpackPakReadError(String, Error),
    PakUnpackOutputPathNotDir(String),
    PakUnpackCanNotCreateOutputPath(String, Error),
    PakWriteIndexFileFail(String, Error),
    PakReadIndexFileFail(String, Error),
    PakPackReadResourceError(PathBuf, Error),
    PakPackWriteFileError(String, Error),
    // resource_id, offset
    PakPackResourceOffsetOverflow(u16, usize),
}