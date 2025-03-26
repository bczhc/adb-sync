use bincode::{Decode, Encode};
use std::path::PathBuf;

/// # Protocol
///
/// TODO

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Message {
    Ok = 1,
    StartIndexing(SendConfig),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct SendConfig {
    pub path: PathBuf,
    /// Skip failures while indexing
    pub skip_failed: bool,
}

pub const MAGIC: &[u8; 11] = b"sync-stream";
