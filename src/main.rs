extern crate rusqlite;

use std::env;
use std::fs;
use std::path::Path;
use rusqlite::{Connection, params};

const DB_PATH: &str = "file_details.db";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: <command> [path]");
        eprintln!("Usage: index [path] #index all files into sqlite db file file_details.db");
        eprintln!("Usage: duplicates # list duplicates from db");
        eprintln!("Usage: largefiles # list 100 largest files from db");
        eprintln!("Author: Senthil Nayagam");
        return;
    }

    match args[1].as_str() {
        "index" => {
            if args.len() < 3 {
                eprintln!("Usage: index <path>");
                return;
            }
            let path = &args[2];
            index_files(path);
        }
        "duplicates" => list_duplicates(),
        "largefiles" => list_large_files(),
        _ => eprintln!("Unknown command. Use 'index', 'duplicates', or 'largefiles'."),
    }
}

fn index_files<P: AsRef<Path>>(path: P) {
    let conn = Connection::open(DB_PATH).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE,
            name TEXT,
            size INTEGER,
            extension TEXT
        )",
        [],
    ).unwrap();

    let mut file_count = 0;
    index_recursive(&path, &conn, &mut file_count);

    println!("Indexing completed. Total files indexed: {}", file_count);
}

fn index_recursive<P: AsRef<Path>>(path: P, conn: &Connection, file_count: &mut u32) {
    for entry in fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();

        if metadata.is_file() {
            *file_count += 1;

            let size = metadata.len() as i64;
            let file_name = entry.file_name().into_string().unwrap();
            let extension = entry.path().extension().and_then(|os| os.to_str()).unwrap_or("").to_string();
            let path = entry.path().to_str().unwrap().to_string();

            conn.execute(
                "INSERT OR REPLACE INTO files (path, name, size, extension) VALUES (?1, ?2, ?3, ?4)",
                params![path, file_name, size, extension],
            ).unwrap();
        } else if metadata.is_dir() {
            index_recursive(entry.path(), conn, file_count);
        }
    }
}


fn list_duplicates() {
    let conn = Connection::open(DB_PATH).unwrap();

    let mut stmt = conn.prepare(
        "SELECT name, size, COUNT(*) FROM files GROUP BY size, name HAVING COUNT(*) > 1 ORDER BY size DESC"
    ).unwrap();

    let duplicates: Vec<(String, i64, i64)> = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let size: i64 = row.get(1)?;
        let count: i64 = row.get(2)?;
        Ok((name, size, count))
    }).unwrap().map(|dup| dup.unwrap()).collect();

    let duplicate_count = duplicates.len();

    if duplicate_count == 0 {
        println!("No duplicates found.");
        return;
    }

    println!("{} duplicates found:", duplicate_count);
    for (name, size, count) in duplicates {
        println!("File: {}, Size: {} appeared {} times:", name, size, count);

        // Fetch and print all file paths for the given name and size
        let mut path_stmt = conn.prepare(
            "SELECT path FROM files WHERE name = ?1 AND size = ?2"
        ).unwrap();
        let file_paths = path_stmt.query_map(params![name, size], |row| {
            let path: String = row.get(0)?;
            Ok(path)
        }).unwrap();

        for path in file_paths {
            let p = path.unwrap();
            println!("\t{}", p);
        }
    }
}


fn list_large_files() {
    let conn = Connection::open(DB_PATH).unwrap();

    let mut stmt = conn.prepare(
        "SELECT name, path, size FROM files ORDER BY size DESC LIMIT 100"
    ).unwrap();

    let files = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let path: String = row.get(1)?;
        let size: i64 = row.get(2)?;
        Ok((name, path, size))
    }).unwrap();

    println!("Top 100 files by size:");
    for file in files {
        let (name, path, size) = file.unwrap();
        println!("{} - {} - {} bytes", name, path, size);
    }
}
