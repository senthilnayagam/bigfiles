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

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        if metadata.is_file() {
            let size = metadata.len() as i64;
            let file_name = entry.file_name().into_string().unwrap();
            let extension = entry.path().extension().and_then(|os| os.to_str()).unwrap_or("").to_string();
            let path = entry.path().to_str().unwrap().to_string();

            conn.execute(
                "INSERT OR REPLACE INTO files (path, name, size, extension) VALUES (?1, ?2, ?3, ?4)",
                params![path, file_name, size, extension],
            ).unwrap();
        }
    }

    println!("Indexing completed.");
}

fn list_duplicates() {
    let conn = Connection::open(DB_PATH).unwrap();

    let mut stmt = conn.prepare(
        "SELECT name, size, COUNT(*) FROM files GROUP BY size, name HAVING COUNT(*) > 1 ORDER BY size DESC"
    ).unwrap();

    let duplicates = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let size: i64 = row.get(1)?;
        let count: i64 = row.get(2)?;
        Ok((name, size, count))
    }).unwrap();

    println!("Found Duplicates:");
    for dup in duplicates {
        let (name, size, count) = dup.unwrap();
        println!("File: {}, Size: {} appeared {} times", name, size, count);
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
