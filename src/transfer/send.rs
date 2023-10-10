use std::ffi::OsStr;
use std::io::{stdin, stdout};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use adb_sync::transfer::Stream;
use adb_sync::{bincode_deserialize_compress, cli_args, enable_backtrace};

fn main() -> anyhow::Result<()> {
    enable_backtrace();
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: Command <android-dir>");
        println!("Stdin: send list");
        return Ok(());
    }

    let android_dir = Path::new(&args[0]);

    let send_list: Vec<Vec<u8>> = bincode_deserialize_compress(stdin())?;

    let mut writer = zstd::Encoder::new(stdout(), 1)?;
    writer.multithread(num_cpus::get() as u32)?;
    writer.include_checksum(true)?;

    let mut stream = Stream::new(&mut writer);
    for b in send_list {
        let relative_path = Path::new(OsStr::from_bytes(&b));
        if relative_path.components().count() == 0 {
            continue;
        }
        let path = android_dir.join(relative_path);

        stream.append_file(relative_path, &path)?;
    }
    drop(stream);

    writer.finish()?;

    Ok(())
}
