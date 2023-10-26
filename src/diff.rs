use std::fs::File;
use std::io::stdout;
use std::path::Path;

use adb_sync::{bincode_deserialize_compress, bincode_serialize_compress, cli_args, enable_backtrace, Entry, generate_send_list};

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

    let send_list = generate_send_list(&entries, dest_dir)?;

    eprintln!("Send list count: {}", send_list.len());
    bincode_serialize_compress(&mut stdout(), send_list)?;

    Ok(())
}
