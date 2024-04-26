use std::io::{Read, Write};

use bincode::error::DecodeError;
use bincode::{Decode, Encode};
use readwrite::ReadWrite;

use crate::bincode_config;

mod android;
mod host;
mod protocol;

// By default, consoles use line-buffering
// so after each `write` call we use `flush()`.
pub struct ReadWriteFlush<R: Read, W: Write>(pub ReadWrite<R, W>);

impl<R: Read, W: Write> Read for ReadWriteFlush<R, W> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl<R: Read, W: Write> Write for ReadWriteFlush<R, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = self.0.write(buf)?;
        self.0.flush()?;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

pub trait WriteBincode<W: Write> {
    fn write_bincode<E: Encode>(&mut self, obj: &E) -> Result<usize, bincode::error::EncodeError>;
}

impl<W: Write> WriteBincode<W> for W {
    fn write_bincode<E: Encode>(&mut self, obj: &E) -> Result<usize, bincode::error::EncodeError> {
        bincode::encode_into_std_write(obj, self, bincode_config())
    }
}

pub trait ReadBincode<R: Read> {
    fn read_bincode<D: Decode>(&mut self) -> Result<D, DecodeError>;
}

impl<R: Read> ReadBincode<R> for R {
    fn read_bincode<D: Decode>(&mut self) -> Result<D, DecodeError> {
        bincode::decode_from_std_read(self, bincode_config())
    }
}
