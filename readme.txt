ported the python code to rust for portability and having single binary

a) index files in folder in sqlite database

b) detect basic duplicates with filename and size match

c) list large files



todo:

a) list instructions if no params passed

b) if no duplicates found , either put 0 duplicates found or no duplicates found

c) version number, and my name



build
cargo run
cargo build
cargo check

cargo build --release