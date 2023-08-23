extern crate crc as crc_lib;

use crc_lib::{Digest, Width};
use std::io;
use std::io::Write;

pub struct DigestWriter<'a, 'b, W>
where
    W: Width,
{
    digest: &'a mut Digest<'b, W>,
}

impl<'a, 'b> Write for DigestWriter<'a, 'b, u32> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, 'b> DigestWriter<'a, 'b, u32> {
    pub fn new(digest: &'a mut Digest<'b, u32>) -> DigestWriter<'a, 'b, u32> {
        Self { digest }
    }
}

impl<'a, 'b> Write for DigestWriter<'a, 'b, u64> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, 'b> DigestWriter<'a, 'b, u64> {
    pub fn new(digest: &'a mut Digest<'b, u64>) -> DigestWriter<'a, 'b, u64> {
        Self { digest }
    }
}

pub mod write {
    extern crate crc as crc_lib;

    use crc_lib::{Digest, Width};
    use std::io::Write;

    pub struct CrcFilter<'a, 'b, W, Wr>
    where
        W: Width,
        Wr: Write,
    {
        digest: &'a mut Digest<'b, W>,
        writer: &'a mut Wr,
    }

    impl<'a, 'b, Wr> CrcFilter<'a, 'b, u32, Wr>
    where
        Wr: Write,
    {
        pub fn new(digest: &'a mut Digest<'b, u32>, writer: &'a mut Wr) -> Self {
            Self { digest, writer }
        }
    }

    impl<'a, 'b, Wr> Write for CrcFilter<'a, 'b, u32, Wr>
    where
        Wr: Write,
    {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let write_size = self.writer.write(buf)?;
            self.digest.update(&buf[..write_size]);
            Ok(write_size)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.writer.flush()
        }
    }
}
