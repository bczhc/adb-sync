#![feature(try_blocks)]

use std::io;
use std::io::stdout;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use adb_sync::{bincode_serialize_compress, cli_args, enable_backtrace, Entry};

fn main() -> anyhow::Result<()> {
    enable_backtrace();
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: command <android-dir>");
        return Ok(());
    }
    let dir = Path::new(&args[0]);

    let walk_dir = jwalk::WalkDir::new(dir).skip_hidden(false);
    let mut entries = Vec::new();
    for x in walk_dir {
        let Ok(entry) = x else {
            eprintln!("Failed to index: {:?}", x);
            continue;
        };
        if entry.file_type.is_dir() {
            // don't send directories
            continue;
        }
        let result: io::Result<Entry> = try {
            let metadata = entry.metadata()?;
            let path = entry.path();
            let relative_path = pathdiff::diff_paths(&path, dir).unwrap();
            Entry {
                path_bytes: relative_path.as_os_str().as_bytes().to_vec(),
                size: metadata.len(),
                modified: metadata.modified()?,
            }
        };
        match result {
            Ok(e) => {
                entries.push(e);
            }
            Err(e) => {
                eprintln!("Error: {:?}", (e, entry));
            }
        }
    }
    eprintln!("Total entries: {}", entries.len());

    let mut stdout = stdout().lock();
    bincode_serialize_compress(&mut stdout, entries)?;

    Ok(())
}
