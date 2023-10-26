#![feature(try_blocks)]

use std::io::stdout;
use std::path::Path;

use adb_sync::{bincode_serialize_compress, cli_args, enable_backtrace, index_dir, Entry};

fn main() -> anyhow::Result<()> {
    enable_backtrace();
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: command <android-dir>");
        return Ok(());
    }
    let dir = Path::new(&args[0]);

    let entries = index_dir(dir)?;
    eprintln!("Total entries: {}", entries.len());

    let mut stdout = stdout().lock();
    bincode_serialize_compress(&mut stdout, entries)?;

    Ok(())
}
