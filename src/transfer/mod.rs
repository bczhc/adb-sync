extern crate crc as crc_lib;

use std::fs::File;
use std::io;
use std::io::Write;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::time::SystemTime;

use ::crc::Crc;
/// Stream structure: \[ header length | header | file data | checksum of header and file data \]
use bincode::{Decode, Encode};
use byteorder::{WriteBytesExt, LE};

use crate::bincode_config;

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
}

pub fn create_crc() -> Crc<u32> {
    crc_lib::Crc::<u32>::new(&crc_lib::CRC_32_CKSUM)
}

impl<W: Write> Drop for Stream<W> {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}
