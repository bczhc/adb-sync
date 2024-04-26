use std::io::Read;
use std::io::Write;
use std::path::Path;

use anyhow::anyhow;
use bytesize::ByteSize;

use crate::send_stream::receive;
use crate::stream::protocol::{Message, SendConfig, MAGIC};
use crate::stream::{ReadBincode, WriteBincode};
use crate::{generate_send_list, Entry};

pub fn start<S: Read + Write>(
    mut stream: S,
    send_config: SendConfig,
    dest_dir: &Path,
) -> anyhow::Result<()> {
    macro_rules! check_ok {
        () => {
            if (stream.read_bincode::<Message>()? != Message::Ok) {
                return Err(anyhow!("Android is not OK!"));
            }
        };
    }

    stream.write_all(MAGIC)?;
    stream.write_bincode(&Message::StartIndexing(send_config))?;
    check_ok!();

    let entries = stream.read_bincode::<Vec<Entry>>()?;
    println!(
        "Entries: {}, {}",
        entries.len(),
        ByteSize(entries.iter().map(|x| x.size).sum::<u64>()).to_string_as(true)
    );
    check_ok!();

    let send_list = generate_send_list(entries, dest_dir)?;
    println!(
        "Send list: {}, {}",
        send_list.len(),
        ByteSize(send_list.iter().map(|x| x.size).sum::<u64>()).to_string_as(true)
    );
    stream.write_bincode(&send_list)?;
    check_ok!();

    receive(&mut stream, dest_dir)?;
    check_ok!();
    println!("Done!");

    Ok(())
}
