use std::io::Read;
use std::io::Write;
use std::path::Path;

use anyhow::anyhow;

use crate::send_stream::receive;
use crate::stream::protocol::{Message, SendConfig, MAGIC};
use crate::stream::{ReadBincode, WriteBincode};
use crate::{bincode_deserialize_compress, bincode_serialize_compress, generate_send_list, Entry};

pub fn start<S: Read + Write>(
    mut stream: S,
    send_config: SendConfig,
    dest_dir: &Path,
) -> anyhow::Result<()> {
    stream.write_all(MAGIC)?;
    stream.write_bincode(&Message::StartIndexing(send_config))?;

    macro_rules! check_ok {
        () => {
            if (stream.read_bincode::<Message>()? != Message::Ok) {
                return Err(anyhow!("Android is not OK!"));
            }
        };
    }

    check_ok!();
    let entries = bincode_deserialize_compress::<_, Vec<Entry>>(&mut stream)?;
    println!("Entries: {}", entries.len());

    let send_list = generate_send_list(&entries, dest_dir)?;
    bincode_serialize_compress(&mut stream, &send_list)?;
    check_ok!();

    receive(&mut stream, dest_dir)?;

    check_ok!();
    println!("Done!");

    Ok(())
}
