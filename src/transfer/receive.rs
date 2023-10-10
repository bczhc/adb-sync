extern crate crc as crc_lib;

use std::ffi::OsStr;
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::{fs, io};

use anyhow::anyhow;
use byteorder::{ReadBytesExt, LE};
use filetime::FileTime;

use adb_sync::transfer::{create_crc, FileType, Header};
use adb_sync::{bincode_config, cli_args, enable_backtrace};

use crate::crc::write::CrcFilter;

mod crc;

fn main() -> anyhow::Result<()> {
    enable_backtrace();
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: Command <extract-dir>");
        return Ok(());
    }

    let extract_dir = Path::new(&args[0]);

    let mut reader = stdin().lock();
    loop {
        let header_length = reader.read_u32::<LE>();
        let header_length = match header_length {
            Ok(l) => l,
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    // reach the end
                    break;
                } else {
                    return Err(anyhow!("Read error: {}", e));
                }
            }
        };

        let mut header_buf = vec![0_u8; header_length as usize];
        reader.read_exact(&mut header_buf)?;
        let (header, _): (Header, _) = bincode::decode_from_slice(&header_buf, bincode_config())?;
        let path = Path::new(OsStr::from_bytes(&header.path));
        let dest_path = extract_dir.join(path);
        match header.file_type {
            FileType::RegularFile => {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut dest_file = File::options()
                    .create(true)
                    .truncate(true)
                    .read(true)
                    .write(true)
                    .open(&dest_path)?;
                let mut file_reader = reader.by_ref().take(header.file_size);

                let crc = create_crc();
                let mut digest = crc.digest();
                digest.update(&header_buf);

                let mut crc_filter = CrcFilter::new(&mut digest, &mut dest_file);
                io::copy(&mut file_reader, &mut crc_filter)?;
                crc_filter.flush()?;

                let checksum = digest.finalize();
                let stored_checksum = reader.read_u32::<LE>()?;
                if checksum != stored_checksum {
                    panic!("Checksum mismatch! {}", path.display());
                }
            }
            FileType::Directory => {
                fs::create_dir_all(&dest_path)?;

                let crc = create_crc();
                let mut digest = crc.digest();
                digest.update(&header_buf);
                let checksum = digest.finalize();
                let stored_checksum = reader.read_u32::<LE>()?;
                if checksum != stored_checksum {
                    panic!("Checksum mismatch! {}", path.display());
                }
            }
        }
        filetime::set_file_mtime(&dest_path, FileTime::from(header.mtime))?;
        println!("{}", dest_path.display());
    }

    Ok(())
}
