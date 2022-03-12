pub enum PakError {
    Unknown,
    // actual, expected
    VersionSizeNotEnough(usize, usize),
    // version
    UnsupportedVersion(u32),
    // expected, actual
    VersionMisMatch(u32, u32),
    // expected, actual
    V4HeaderSizeNotEnough(usize, usize),
    // expected, actual
    V5HeaderSizeNotEnough(usize, usize)
}