extern crate rusqlite;

use std::env;
use std::fs;
use std::path::Path;

use rusqlite::{Connection, params}; //sqlite

use prettytable::{Table, Row, Cell, format}; //table display
use prettytable::row;
use prettytable::cell;

use indicatif::{ProgressBar, ProgressStyle}; // progressbar

use warp::{Filter}; // , Reply // web framework
use warp::http::StatusCode;

use std::net::{UdpSocket}; //, SocketAddrV4, Ipv4Addr};

const DB_PATH: &str = "bigfiles.db";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: bigfiles <command> [param]");
        eprintln!("bigfiles index [path] #index all files into sqlite db file file_details.db");
        eprintln!("bigfiles duplicates # list duplicates from db");
        eprintln!("bigfiles largefiles # list 100 largest files from db");
        eprintln!("bigfiles server # web interface to list and search files\n");
        eprintln!("Author: Senthil Nayagam");
       list_version();
        
        return;
    }

    match args[1].as_str() {
        "index" => {
            if args.len() < 3 {
                eprintln!("Usage: index <path>");
                return;
            }
            let path = &args[2];
            //index_files(path);
            index_files_recursive(path);
        }
        "duplicates" => list_duplicates(),
        "largefiles" => list_large_files(),
        "version" => list_version(),
        "server" => {
            start_server();
        },
        _ => eprintln!("Unknown command. Use 'index', 'duplicates', 'largefiles' or 'server'."),
    }
}




fn count_files<P: AsRef<Path>>(path: P) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(&path).expect("Failed to read directory") {
        let entry_path = entry.expect("Failed to read entry").path();
        if entry_path.is_dir() {
            count += count_files(&entry_path);
        } else {
            count += 1;
        }
    }
    count
}

fn index_files_recursive<P: AsRef<Path>>(path: P) {
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

    let total_files = count_files(&path);
    let bar = ProgressBar::new(total_files as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    fn recurse_and_index<P: AsRef<Path>>(path: P, conn: &Connection, bar: &ProgressBar) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                recurse_and_index(entry.path(), conn, bar);
            } else if metadata.is_file() {
                let size = metadata.len() as i64;
                let file_name = entry.file_name().into_string().unwrap();
                let extension = entry.path().extension().and_then(|os| os.to_str()).unwrap_or("").to_string();
                let path = entry.path().to_str().unwrap().to_string();

                conn.execute(
                    "INSERT OR REPLACE INTO files (path, name, size, extension) VALUES (?1, ?2, ?3, ?4)",
                    params![path, file_name, size, extension],
                ).unwrap();
                bar.inc(1);
            }
        }
    }

    recurse_and_index(&path, &conn, &bar);

    bar.finish();
    println!("Indexing completed.");
}



fn list_version() {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
    eprintln!("Version: {}",CARGO_PKG_VERSION.unwrap_or("NOT_FOUND"));
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


    let mut table = Table::new();
    table.add_row(row!["Filename", "Size (in bytes)", "Count"]);


    for dup in duplicates {
        match dup {
            Ok((name, size, count)) => {
               // println!("Found duplicate: Name: {}, Size: {}, Count: {}", name, size, count);
                //table.add_row(row![name, size, count]);
                table.add_row(Row::new(vec![
                    Cell::new(&name),
                    Cell::new(&size.to_string()),
                    Cell::new(&count.to_string()),
                ]));
            }
            Err(e) => {
                println!("Error processing a duplicate entry: {}", e);
            }
        }
    }
   // table.printstd();
    if table.len() > 1 {
        println!("Found Duplicates:");
        table.printstd();
    } else {
        println!("No duplicates found.");
    }
}







fn list_large_files() {
    let conn = Connection::open(DB_PATH).unwrap();
    let mut stmt = conn.prepare("SELECT path, name, size FROM files ORDER BY size DESC LIMIT 50").unwrap();
    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);
    table.add_row(row!["Path", "Filename", "Size (in bytes)"]); // Header row

    stmt.query_map([], |row| {
        let path: String = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let size: i64 = row.get(2).unwrap();
       // println!("{}\t{}\t{}", path,name,size);

        table.add_row(Row::new(vec![
            Cell::new(&path),
            Cell::new(&name),
            Cell::new(&size.to_string()),
        ]));

        Ok(())
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

    table.printstd();
}





async fn index_web() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        "you can try \n /duplicates \n /largefiles",
        StatusCode::OK,
    ))
}

async fn list_duplicates_web() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        "Duplicates List - This is a placeholder for now.",
        StatusCode::OK,
    ))
}

async fn list_large_files_web() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        "Large Files List - This is a placeholder for now.",
        StatusCode::OK,
    ))
}

fn start_server() {
    //let index = warp::path("").and_then(index_web);
    if let Some(ip) = get_local_ip() {
        println!("Local IP: {}", ip);
        let port = 3030; // Adjust the port as necessary
        let url = format!("http://{}:{}/", ip, port);
        println!("Starting the server on {}", url);
        println!("Scan the QR code below for the URL:");
        //generate_qr3(&url);
        qr2term::print_qr(&url).unwrap();

    } else {
        println!("Failed to fetch the local IP");
    }

     // GET /
    let index = warp::path::end().and_then( index_web);
    let duplicates = warp::path("duplicates").and_then(list_duplicates_web);
    let large_files = warp::path("largefiles").and_then(list_large_files_web);

    let routes = warp::get().and(index.or(large_files).or(duplicates));
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    });
}




fn get_local_ip() -> Option<String> {
    // We bind to the address below to figure out what our local IP is.
    let socket = UdpSocket::bind("0.0.0.0:0").expect("binding to local address failed");
    // This will not actually establish a connection, but will choose the correct local address to use.
    socket.connect("8.8.8.8:80").expect("connection to 8.8.8.8:80 failed");
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

