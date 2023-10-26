# Bigfiles

Ported the initial Python code to Rust for portability and having a single binary.

## Features

1. **Indexing**: Index files in a folder in the SQLite database.
2. **Duplicate Detection**: Detect basic duplicates with filename and size match.
3. **Large Files**: List large files.

## Progress

### Done

- [x] List instructions if no params passed.
- [x] Display message if no duplicates found (either "0 duplicates found" or "no duplicates found").
- [x] Version number and author information.
- [x] find local ip address for accessing it from other computers in the network
- [x]  generate QRcode for the local ip in terminal
- [x] Indexing can be time-consuming. Showing some progress, like a total count or a progress bar, would be better.

### Work in Progress

- [ ] run a simple web app which can list duplicate files and large files and maybe a few sorting or search function via web interface

### Todo

- [ ] Compute and store file SHA in the files table. This is optional as it can be compute-intensive.

- [ ] Display results in a tabular column.
- [ ] Export file list with details as CSV.



## Build Commands

```bash
cargo run
cargo build
cargo check
cargo clean
cargo build --release
cp .\target\release\bigfiles.exe C:\Users\senth\bin\
```


Optimizations
For further optimizations, check out min-sized-rust https://github.com/johnthagen/min-sized-rust
```bash
cargo bloat
cargo bloat --time -j 1
```


# Authors
Author: Senthil Nayagam
Co-Author: ChatGPT4

### Nice to Haves
Cross-compile and release on Windows, Linux, Mac (Intel), and Mac (Apple Silicon).