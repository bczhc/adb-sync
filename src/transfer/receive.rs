#![feature(try_blocks)]

extern crate crc as crc_lib;

use std::io::{stdin, Read, Write};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use byteorder::ReadBytesExt;

use adb_sync::transfer::receive;
use adb_sync::{cli_args, enable_backtrace, TryReadExact};

mod crc;

fn main() -> anyhow::Result<()> {
    enable_backtrace();
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: Command <dest-dir>");
        return Ok(());
    }

    let dest_dir = Path::new(&args[0]);

    let mut reader = stdin().lock();
    receive(&mut reader, dest_dir)?;

    Ok(())
}
