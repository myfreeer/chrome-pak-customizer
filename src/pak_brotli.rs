use std::io::{Result, Write};

use brotli_decompressor::BrotliDecompress;

struct Counter {
    count: usize,
}

impl Write for Counter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let len = buf.len();
        self.count += len;
        Ok(len)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// Update: seems impossible to get uncompressed size without decompression
// https://github.com/google/brotli/issues/861
// https://github.com/google/brotli/issues/809
pub fn brotli_calculate_decompressed_size(buf: &[u8]) -> u64 {
    let mut counter = Counter { count: 0 };
    let mut slice: &[u8] = buf.as_ref();
    if let Err(err) = BrotliDecompress(&mut slice, &mut counter) {
        println!("brotli_calculate_decompressed_size: {:?}", err)
    }
    counter.count as u64
}
