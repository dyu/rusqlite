extern crate rusqlite;
use rusqlite::{Connection, Result, params};
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
    name TEXT    NOT NULL
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
    let prep_stmt = 0 != (flags & 1);
    
    let iterations: usize = env_as_num("ITERATIONS", 5000);
    
    let conn = Connection::open("target/db.sqlite")?;
    conn.exec(DDL)?;
    
    let mut buf_id = Buffer::new();
    
    let start = Instant::now();
    
    if prep_stmt {
        let mut stmt = conn.prepare(
            "INSERT INTO bench_simple (id, ts, name) VALUES (?1, ?2, ?3)",
        )?;
        for idx in 0..iterations {
             stmt.execute(params!(idx as i64, get_epoch_seconds(), buf_id.format(idx)))?;
        }
    } else {
        for idx in 0..iterations {
            conn.execute(
                "INSERT INTO bench_simple (id, ts, name) VALUES (?1, ?2, ?3)",
                params!(idx as i64, get_epoch_seconds(), buf_id.format(idx)),
            )?;
        }
    }
    
    let elapsed = start.elapsed().as_millis();
    let ops_per_sec = (iterations * 1000) / elapsed as usize;
    eprintln!("prep_stmt: {prep_stmt} | iterations: {iterations} | elapsed time: {elapsed} ms | ops/sec: {ops_per_sec}");
    
    Ok(())
}
