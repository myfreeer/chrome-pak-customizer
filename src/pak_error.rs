use std::io::Error;
use std::num::ParseIntError;

use crate::pak_index::PakIndexStatus;

pub enum PakError {
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
    PakWriteFileFail(Error),
    // buffer length, offset
    PakAliasOffsetOverflow(usize, usize),
    // actual, expected
    PakAliasSizeNotEnough(usize, usize),
    PakNotChromiumBrotli,
    PakChromiumBrotliSizeNotEnough(usize),
    PakIndexParseError(String),
    PakIndexUnknownTag(String),
    // status, key, value
    PakIndexUnknownProperty(PakIndexStatus, String, String),
    // status, key, value
    PakIndexInvalidProperty(PakIndexStatus, String, String),
    // version str, err
    PakIndexBadVersion(String, ParseIntError),
    PakIndexBadResourceId(String, ParseIntError),
    // version
    PakIndexAliasNotSupported(u32),
    // key, value, err
    PakIndexAliasBadResourceId(String, String, ParseIntError),
    // key, value, err
    PakIndexAliasBadEntryIndex(String, String, ParseIntError),
    PakIndexUnknownAction(PakIndexStatus, String)
}