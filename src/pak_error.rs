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
}