extern crate rusqlite;
use rusqlite::{Connection, Result};
use std::ffi::{CStr};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use itoa::{Buffer};

const DDL: &'static CStr = c"
PRAGMA journal_mode=WAL;
PRAGMA synchronous=OFF;
PRAGMA page_size=4096;
PRAGMA journal_size_limit=16384;
PRAGMA wal_autocheckpoint=4000;
PRAGMA auto_vacuum=INCREMENTAL;
PRAGMA foreign_keys=ON;
PRAGMA optimize=0x10002;
CREATE TABLE IF NOT EXISTS bench_simple (
    id INTEGER NOT NULL CONSTRAINT bench_simple_pkey PRIMARY KEY,
    ts INTEGER NOT NULL,
    name TEXT NOT NULL
);
";

fn arg_as_num<T: std::str::FromStr>(nth: usize, def: T) -> T {
    std::env::args().nth(nth)
        .and_then(|e| e.parse::<T>().ok())
        .unwrap_or(def)
}

fn env_as_num<T: std::str::FromStr>(key: &str, def: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|e| e.parse::<T>().ok())
        .unwrap_or(def)
}

fn get_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::remove_file("target/db.sqlite");
    let _ = std::fs::remove_file("target/db.sqlite-shm");
    let _ = std::fs::remove_file("target/db.sqlite-wal");
    
    let flags: u8 = arg_as_num(1, 0);
    let begin_end = 0 != (flags & 1);
    
    let iterations: usize = env_as_num("ITERATIONS", 5000);
    
    let conn = Connection::open("target/db.sqlite")?;
    conn.exec(DDL)?;
    
    let mut buf_id = Buffer::new();
    let mut buf_ts = Buffer::new();
    let mut buf = String::new();
    
    let start = Instant::now();
    
    if begin_end {
        buf.push_str("BEGIN;")
    }
    buf.push_str("INSERT INTO bench_simple (id, ts, name) VALUES (");
    for idx in 0..iterations {
        let id = buf_id.format(idx);
        let ts = buf_ts.format(get_epoch_seconds());
        if idx != 0 {
            buf.push_str(",(");
        }
        buf.push_str(id);
        buf.push(',');
        buf.push_str(ts);
        buf.push_str(",'");
        buf.push_str(id);
        buf.push_str("')");
    }
    if begin_end {
        buf.push_str(";END");
    }
    buf.push_str(";\0");
    
    conn.exec_ptr(buf.as_ptr())?;
    
    let elapsed = start.elapsed().as_millis();
    let ops_per_sec = (iterations * 1000) / elapsed as usize;
    eprintln!("begin_end: {begin_end} | iterations: {iterations} | elapsed time: {elapsed} ms | ops/sec: {ops_per_sec}");
    
    Ok(())
}
