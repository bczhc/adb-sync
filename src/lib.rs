#![feature(try_blocks)]

use std::env::args;
use std::io::{Read, Write};
use std::path::Path;
use std::time::SystemTime;
use std::{env, io};

use bincode::config::Configuration;
use bincode::{Decode, Encode};

use crate::unix_path::UnixPath;

pub mod crc;
mod send_stream;
pub mod stream;
pub mod unix_path;

#[derive(Encode, Decode)]
pub struct Entry {
    pub path: UnixPath,
    pub size: u64,
    pub modified: SystemTime,
}

pub fn cli_args() -> Vec<String> {
    args().skip(1).collect::<Vec<_>>()
}

pub fn bincode_config() -> Configuration {
    bincode::config::standard().with_variable_int_encoding()
}

pub fn enable_backtrace() {
    env::set_var("RUST_BACKTRACE", "1");
}

pub trait TryReadExact {
    /// Read exact data
    ///
    /// This function blocks. It reads exact data, and returns bytes it reads. The return value
    /// will always be the buffer size until it reaches EOF.
    ///
    /// When reaching EOF, the return value will be less than the size of the given buffer,
    /// or just zero.
    ///
    /// This simulates C function `fread`.
    fn try_read_exact(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

impl<R> TryReadExact for R
where
    R: Read,
{
    fn try_read_exact(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read = 0_usize;
        loop {
            let result = self.read(&mut buf[read..]);
            match result {
                Ok(r) => {
                    if r == 0 {
                        return Ok(read);
                    }
                    read += r;
                    if read == buf.len() {
                        return Ok(read);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub fn index_dir<P: AsRef<Path>>(dir: P, skip_failed: bool) -> io::Result<Vec<Entry>> {
    let walk_dir = jwalk::WalkDir::new(dir.as_ref()).skip_hidden(false);
    let mut entries = Vec::new();
    for x in walk_dir {
        let Ok(entry) = x else {
            if skip_failed {
                eprintln!("Failed to index: {:?}", x);
                continue;
            } else {
                return Err(io::Error::from(x.err().unwrap()));
            }
        };
        if entry.file_type.is_dir() {
            // don't send directories
            continue;
        }
        let result: io::Result<Entry> = try {
            let metadata = entry.metadata()?;
            let path = entry.path();
            let relative_path = pathdiff::diff_paths(&path, dir.as_ref()).unwrap();
            Entry {
                path: relative_path.into(),
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
    Ok(entries)
}

pub fn generate_send_list<P: AsRef<Path>>(
    entries: &[Entry],
    dest_dir: P,
) -> io::Result<Vec<UnixPath>> {
    let mut send_list = Vec::new();
    for e in entries {
        let path = &e.path.0;
        let dest_file = dest_dir.as_ref().join(path);
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
            send_list.push(path.into())
        }
    }
    Ok(send_list)
}
