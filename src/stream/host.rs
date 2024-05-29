use std::io::Read;
use std::io::Write;
use std::path::Path;

use anyhow::anyhow;
use bytesize::ByteSize;
use colored::Colorize;
use log::info;

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

    info!("{}", "Start sending...".cyan().bold());
    stream.write_all(MAGIC)?;
    stream.write_bincode(&Message::StartIndexing(send_config))?;
    check_ok!();

    info!("{}", "Indexing...".cyan().bold());
    let entries = stream.read_bincode::<Vec<Entry>>()?;
    info!(
        "{}",
        format!(
            "Entries: {}, {}",
            entries.len(),
            ByteSize(entries.iter().map(|x| x.size).sum::<u64>()).to_string_as(true)
        )
        .cyan()
        .bold()
    );
    check_ok!();

    info!("{}", "Generating send list...".cyan().bold());
    let send_list = generate_send_list(entries, dest_dir)?;
    info!(
        "{}",
        format!(
            "Send list: {}, {}",
            send_list.len(),
            ByteSize(send_list.iter().map(|x| x.size).sum::<u64>()).to_string_as(true)
        )
        .cyan()
        .bold()
    );
    stream.write_bincode(&send_list)?;
    check_ok!();

    info!("{}", "Receiving...".cyan().bold());
    receive(&mut stream, dest_dir)?;
    check_ok!();
    info!("{}", "Done!".cyan().bold());

    Ok(())
}
