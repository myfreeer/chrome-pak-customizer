use brotli_decompressor::BrotliDecompress;

// Update: seems impossible to get uncompressed size without decompression
// https://github.com/google/brotli/issues/861
// https://github.com/google/brotli/issues/809
pub fn brotli_calculate_decompressed_size(buf: &[u8]) -> u64 {
    // TODO: maybe empty Write?
    let mut vec = Vec::new();
    let mut slice: &[u8] = buf.as_ref();
    let result = BrotliDecompress(&mut slice, &mut vec);
    match result {
        Ok(_) => {}
        Err(err) => println!("brotli_calculate_decompressed_size: {:?}", err)
    }
    vec.len() as u64
}