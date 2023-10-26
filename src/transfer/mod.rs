extern crate crc as crc_lib;

use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

use ::crc::Crc;
use anyhow::anyhow;
use bincode::{Decode, Encode};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use filetime::FileTime;

use crate::transfer::crc::write::CrcFilter;
use crate::{bincode_config, TryReadExact};

pub mod tcp;

mod crc;

#[derive(Encode, Decode)]
pub struct Header {
    pub path: Vec<u8>,
    pub file_type: FileType,
    pub mtime: SystemTime,
    pub file_size: u64,
}

#[derive(Encode, Decode, Copy, Clone)]
pub enum FileType {
    RegularFile,
    Directory,
}

pub struct Stream<W: Write> {
    writer: W,
}

impl<W> Stream<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

/// Send stream
///
/// - Stream structure:
///   \[ Record1 | Record2 | ... \]
///
/// - Record structure:
///   \[ HeaderLength (u32) | Header | FileContent | Checksum (u32) \]
///   
///   When `HeaderLength` is 0xFFFFFFFF, it indicates EOF.
impl<W> Stream<W>
where
    W: Write,
{
    pub fn append_file<P: AsRef<Path>>(&mut self, header_path: P, file_path: P) -> io::Result<()> {
        let header_path = header_path.as_ref();
        let metadata = file_path.as_ref().symlink_metadata()?;

        let (file_type, file_size) = if metadata.is_file() {
            (FileType::RegularFile, metadata.len())
        } else if metadata.is_dir() {
            (FileType::Directory, 0)
        } else {
            eprintln!("Skip: {}", header_path.display());
            return Ok(());
        };
        let header = Header {
            file_type,
            mtime: metadata.modified()?,
            path: header_path.as_os_str().as_bytes().to_vec(),
            file_size,
        };
        let header_data = bincode::encode_to_vec(header, bincode_config()).unwrap();
        self.writer.write_u32::<LE>(header_data.len() as u32)?;
        self.writer.write_all(&header_data)?;

        match file_type {
            FileType::RegularFile => {
                let crc = create_crc();
                let mut digest = crc.digest();
                digest.update(&header_data);

                let mut crc_filter = crc::write::CrcFilter::new(&mut digest, &mut self.writer);
                let mut file = File::open(file_path)?;
                io::copy(&mut file, &mut crc_filter)?;
                crc_filter.flush()?;
                let checksum = digest.finalize();
                self.writer.write_u32::<LE>(checksum)?;
            }
            FileType::Directory => {
                let crc = crc_lib::Crc::<u32>::new(&crc_lib::CRC_32_CKSUM);
                let mut digest = crc.digest();
                digest.update(&header_data);
                self.writer.write_u32::<LE>(digest.finalize())?;
            }
        }

        Ok(())
    }

    fn write_eof(&mut self) -> io::Result<()> {
        self.writer.write_u32::<LE>(0xFFFFFFFF)
    }
}

pub fn create_crc() -> Crc<u32> {
    crc_lib::Crc::<u32>::new(&crc_lib::CRC_32_CKSUM)
}

impl<W: Write> Drop for Stream<W> {
    fn drop(&mut self) {
        let _ = self.writer.flush();
        let _ = self.write_eof();
        let _ = self.writer.flush();
    }
}

pub fn write_send_list_to_stream<P, W>(
    stream: &mut Stream<W>,
    android_dir: P,
    send_list: &[Vec<u8>],
) -> io::Result<()>
where
    P: AsRef<Path>,
    W: Write,
{
    for b in send_list {
        let relative_path = Path::new(OsStr::from_bytes(b));
        if relative_path.components().count() == 0 {
            continue;
        }
        let path = android_dir.as_ref().join(relative_path);

        stream.append_file(relative_path, &path)?;
    }
    Ok(())
}

pub fn receive<P, R>(mut reader: R, dest_dir: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    R: Read,
{
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
        let dest_path = &dest_dir.as_ref().join(header_path);
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
