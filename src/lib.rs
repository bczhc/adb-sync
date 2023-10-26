use std::env;
use std::env::args;
use std::io::{Read, Write};
use std::time::SystemTime;

use bincode::config::Configuration;
use bincode::{Decode, Encode};

pub mod transfer;

#[derive(Encode, Decode)]
pub struct Entry {
    /// `bincode` doesn't support (de)serializing non-UTF8 `Path`s
    pub path_bytes: Vec<u8>,
    pub size: u64,
    pub modified: SystemTime,
}

pub fn cli_args() -> Vec<String> {
    args().skip(1).collect::<Vec<_>>()
}

pub fn bincode_config() -> Configuration {
    bincode::config::standard().with_variable_int_encoding()
}

pub fn bincode_serialize_compress<W: Write, E: Encode>(
    mut writer: W,
    obj: E,
) -> anyhow::Result<()> {
    let mut encoder = zstd::Encoder::new(&mut writer, 5)?;
    encoder.include_checksum(true)?;
    encoder.multithread(num_cpus::get() as u32)?;
    bincode::encode_into_std_write(obj, &mut encoder, bincode_config())?;
    encoder.finish()?;
    Ok(())
}

pub fn bincode_deserialize_compress<R: Read, D: Decode>(mut reader: R) -> anyhow::Result<D> {
    let mut decoder = zstd::Decoder::new(&mut reader)?;
    Ok(bincode::decode_from_std_read(
        &mut decoder,
        bincode_config(),
    )?)
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
    fn try_read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

impl<R> TryReadExact for R
where
    R: Read,
{
    fn try_read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
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
