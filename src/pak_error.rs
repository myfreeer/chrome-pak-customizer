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
    PakEntryOffsetOverflow(usize, usize),
    // actual, expected
    PakEntrySizeNotEnough(usize, usize),
}