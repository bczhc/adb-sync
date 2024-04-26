use std::io::{Read, Write};

use anyhow::anyhow;

use crate::send_stream::{write_send_list_to_stream, SendStream};
use crate::stream::protocol::{Message, SendConfig, MAGIC};
use crate::stream::{ReadBincode, WriteBincode};
use crate::unix_path::UnixPath;
use crate::{bincode_deserialize_compress, bincode_serialize_compress, index_dir};

pub fn handle_connection<S: Read + Write>(mut stream: S) -> anyhow::Result<()> {
    let mut magic_buf = [0_u8; MAGIC.len()];
    stream.read_exact(&mut magic_buf)?;
    if &magic_buf != MAGIC {
        return Err(anyhow!("Invalid magic: {:?}", magic_buf));
    }
    macro_rules! send_ok {
        () => {
            stream.write_bincode(&Message::Ok)?;
        };
    }

    let send_config: SendConfig;
    // wait for `StartIndexing` directive
    loop {
        if let Message::StartIndexing(config) = stream.read_bincode::<Message>()? {
            send_config = config;
            break;
        }
    }
    send_ok!();

    let entries = index_dir(&send_config.path, true)?;
    bincode_serialize_compress(&mut stream, &entries)?;
    send_ok!();

    let send_list: Vec<UnixPath> = bincode_deserialize_compress(&mut stream)?;
    send_ok!();

    let mut send_stream = SendStream::new(&mut stream);
    write_send_list_to_stream(&mut send_stream, &send_config.path, &send_list, |_, _| {})?;
    drop(send_stream);

    send_ok!();

    Ok(())
}
