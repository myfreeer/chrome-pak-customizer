# Chrome PAK Customizer

`chrome-pak-customizer` is a small Rust CLI for unpacking and repacking Chromium `.pak` resource files.

It supports:

- Chromium PAK v4
- Chromium PAK v5
- Microsoft Edge's v5 variant with `u32` resource IDs

The tool is designed for round-trip editing:

1. unpack a `.pak` file into resource files plus a `pak_index.ini`
2. edit the extracted files
3. repack them back into a `.pak`

## Build

Debug build:

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

## Usage

Unpack a pak file:

```bash
cargo run --release -- -u input.pak output_dir
```

Pack from an extracted directory:

```bash
cargo run --release -- -p output_dir/pak_index.ini output.pak
```

Use Edge v5 mode:

```bash
cargo run --release -- -eu input.pak output_dir
cargo run --release -- -ep output_dir/pak_index.ini output.pak
```

Show help:

```bash
cargo run --release -- -h
```

## Unpack Output

Unpacking creates:

- extracted resource files
- `pak_index.ini`

Typical layout:

```text
output_dir/
  1234.html
  5678.js
  pak_index.ini
```

`pak_index.ini` stores:

- pak version
- encoding
- resource file mapping
- alias entries for v5 files when present

Example:

```ini
[Global]
version=5
encoding=0

[Resources]
1234=1234.html
5678=5678.js:::BrotliCompressed

[Alias]
9999=0
```

`:::BrotliCompressed` means the file should be repacked with Chromium's custom Brotli header. Files without that suffix are packed as raw resources.

## Testing

Run unit tests:

```bash
cargo test --verbose --no-fail-fast
```

Run the round-trip fixture test:

```bash
./script/ci-run.sh
```

The integration test uses real pak files under `test_dir/`, unpacks them into `test_out/`, repacks them, and verifies that the SHA256 hash matches the original.

## Development Notes

- Source code lives in `src/`
- Helper scripts live in `script/`
- CI configuration lives in `.github/workflows/rust.yml`

Important modules:

- `pak_unpack.rs`: pak -> files + `pak_index.ini`
- `pak_pack.rs`: `pak_index.ini` + files -> pak
- `pak_index.rs`: parse and write `pak_index.ini`
- `pak_header.rs`: v4/v5 header handling
- `pak_file_io.rs`: resource file read/write logic

## Limitations

- The CLI is intentionally minimal
- Existing destination files may be overwritten
- Only pak files themselves are packed and unpacked; the tool does not edit resource contents automatically

## License

MIT
