use bincode::{Decode, Encode};
use std::path::PathBuf;

/// # Protocol
///
/// TODO

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum Message {
    Ok,
    StartIndexing(SendConfig),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub struct SendConfig {
    pub path: PathBuf,
}

pub const MAGIC: &[u8; 11] = b"sync-stream";
