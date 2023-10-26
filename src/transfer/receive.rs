#![feature(try_blocks)]

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
use adb_sync::{bincode_config, cli_args, enable_backtrace, TryReadExact};

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
        let mut header_length_buf = [0_u8; 4];
        let size = reader.try_read_exact(&mut header_length_buf)?;
        let header_length = match size {
            0 => {
                return Err(anyhow!("Unexpected EOF before reaching the EOF mark"));
            }
            4 => {
                use byteorder::ByteOrder;
                LE::read_u32(&header_length_buf)
            }
            _ => {
                return Err(anyhow!("Broken stream; only read {} bytes of header", size));
            }
        };

        if header_length == 0xFFFFFFFF {
            // EOF
            // there should be no data to read, test it
            if reader.read_u8().is_ok() {
                return Err(anyhow!("Unexpected data after EOF mark"));
            }
            break;
        }

        let mut header_buf = vec![0_u8; header_length as usize];
        reader.read_exact(&mut header_buf)?;
        let (header, deser_size): (Header, _) =
            bincode::decode_from_slice(&header_buf, bincode_config())?;
        if deser_size != header_length as usize {
            // the used data for deserialization is not the full given data
            // that's unexpected
            panic!("Mismatched header deserialization length");
        }

        let header_path = Path::new(OsStr::from_bytes(&header.path));
        let dest_path = &extract_dir.join(header_path);
        let send_result: anyhow::Result<()> = try {
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
                        .open(dest_path)?;
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
                        Err(anyhow!("Checksum mismatch! {}", header_path.display()))?;
                    }
                }
                FileType::Directory => {
                    fs::create_dir_all(dest_path)?;

                    let crc = create_crc();
                    let mut digest = crc.digest();
                    digest.update(&header_buf);
                    let checksum = digest.finalize();
                    let stored_checksum = reader.read_u32::<LE>()?;
                    if checksum != stored_checksum {
                        Err(anyhow!("Checksum mismatch! {}", header_path.display()))?;
                    }
                }
            }
            filetime::set_file_mtime(dest_path, FileTime::from(header.mtime))?;
            println!("{}", dest_path.display());
        };
        if let Err(e) = send_result {
            // delete the just-failed file/directory and exit
            println!("Cleaning after failure...");
            if dest_path.exists() {
                match header.file_type {
                    FileType::RegularFile => {
                        println!("Remove file: {}", dest_path.display());
                        fs::remove_file(dest_path)?;
                    }
                    FileType::Directory => {
                        // only remove empty directories
                        if fs::read_dir(dest_path)
                            .map(|x| x.count() == 0)
                            .unwrap_or(false)
                        {
                            println!("Remove dir: {}", dest_path.display());
                            fs::remove_dir(dest_path)?;
                        }
                    }
                }
            }
            Err(e)?;
        }
    }

    Ok(())
}
