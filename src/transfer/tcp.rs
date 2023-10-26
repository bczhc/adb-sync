use num_derive::{FromPrimitive, ToPrimitive};

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum Message {
    Finish,
    Eof,
}

pub const STREAM_MAGIC: &[u8; 11] = b"sync-stream";
