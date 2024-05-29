use adb_sync::stream::android::handle_connection;
use adb_sync::stream::ReadWriteFlush;
use readwrite::ReadWrite;
use std::io::{stdin, stdout};

pub fn main() -> anyhow::Result<()> {
    let stream = ReadWriteFlush(ReadWrite::new(stdin(), stdout()));
    handle_connection(stream)?;
    Ok(())
}
