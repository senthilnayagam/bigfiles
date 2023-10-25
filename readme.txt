ported the python code to rust for portability and having single binary

a) index files in folder in sqlite database

b) detect basic duplicates with filename and size match

c) list large files



done:
a) list instructions if no params passed

b) if no duplicates found , either put 0 duplicates found or no duplicates found

c) version number, and my name

todo:

d) compute and store file sha in files table , if asked explicitly as it can be compute intensive

e) indexing takes much time, so showing some progress like total count or some progress bar can be better

f) showing result in tabular column



build
cargo run
cargo build
cargo check
cargo clean


cargo build --release
cp .\target\release\bigfiles.exe C:\Users\senth\bin\

optimisations
https://github.com/johnthagen/min-sized-rust

cargo bloat
cargo bloat --time -j 1


Author: Senthil Nayagam
Co-Author: ChatGPT4



nice to haves
cross compile and release on windows, linux and mac(intel) and mac(apple silicon)

