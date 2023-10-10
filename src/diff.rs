use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::stdout;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use adb_sync::{
    bincode_deserialize_compress, bincode_serialize_compress, cli_args, enable_backtrace, Entry,
};

fn main() -> anyhow::Result<()> {
    enable_backtrace();
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: Command <android-list-file> <dest-dir>");
        return Ok(());
    }

    let list_file = &args[0];
    let dest_dir = Path::new(&args[1]);

    let list_file = File::open(list_file)?;
    let entries: Vec<Entry> = bincode_deserialize_compress(list_file)?;

    let mut send_list = Vec::new();
    for e in entries {
        let path = Path::new(OsStr::from_bytes(&e.path_bytes));
        let dest_file = dest_dir.join(path);
        let send: io::Result<bool> = (|| {
            if !dest_file.exists() {
                return Ok(true);
            }

            let metadata = dest_file.symlink_metadata()?;
            if metadata.len() != e.size {
                return Ok(true);
            }

            if metadata.modified()? != e.modified {
                return Ok(true);
            }

            Ok(false)
        })();
        if send? {
            send_list.push(path.as_os_str().as_bytes().to_vec());
        }
    }

    eprintln!("Send list count: {}", send_list.len());
    bincode_serialize_compress(&mut stdout(), send_list)?;

    Ok(())
}
