use std::io::Error;

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
}