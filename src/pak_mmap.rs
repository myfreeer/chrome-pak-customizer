use std::fs::File;
use std::io;
use std::ops::Deref;
use std::path::Path;

#[cfg(any(unix, windows))]
pub const MMAP_AVAILABLE: bool = true;

#[cfg(not(any(unix, windows)))]
pub const MMAP_AVAILABLE: bool = false;

#[cfg(any(unix, windows))]
pub struct PakMmap {
    map: memmap2::Mmap,
}

#[cfg(not(any(unix, windows)))]
pub struct PakMmap;

#[cfg(any(unix, windows))]
impl Deref for PakMmap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[cfg(not(any(unix, windows)))]
impl Deref for PakMmap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &[]
    }
}

#[cfg(any(unix, windows))]
pub fn map_read_only(path: &Path) -> io::Result<PakMmap> {
    let file = File::open(path)?;
    // The map is read-only and the file handle is not exposed after mapping.
    // Callers must treat externally modifying the file while mapped as invalid.
    let map = unsafe { memmap2::Mmap::map(&file)? };
    Ok(PakMmap { map })
}

#[cfg(not(any(unix, windows)))]
pub fn map_read_only(_path: &Path) -> io::Result<PakMmap> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "memory-mapped IO is not supported on this target",
    ))
}
