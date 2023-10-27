use std::io;
use std::io::{Cursor, Read};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::path::Path;

use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use adb_sync::transfer::receive;
use adb_sync::transfer::tcp::{Message, SendConfigs, STREAM_MAGIC};
use adb_sync::{
    bincode_deserialize_compress, bincode_serialize_compress, cli_args, generate_send_list, Entry,
};

fn main() -> anyhow::Result<()> {
    let args = cli_args();
    if args.is_empty() {
        println!("Usage: cmd <bind-port> <dest-dir>");
        return Ok(());
    }

    let port = args[0].parse::<u16>()?;
    let dest_dir = &args[1];

    let listener = TcpListener::bind(SocketAddrV4::new("0.0.0.0".parse().unwrap(), port))?;
    loop {
        let (socket, addr) = listener.accept()?;
        println!("Connected: {}", addr);
        let result = handle_connection(socket, dest_dir);
        if let Err(e) = result {
            eprintln!("Err: {}", e);
        }
    }
}

fn handle_connection<P: AsRef<Path>>(mut socket: TcpStream, dest_dir: P) -> anyhow::Result<()> {
    let mut magic_buf = [0_u8; STREAM_MAGIC.len()];
    socket.read_exact(&mut magic_buf)?;

    if &magic_buf != STREAM_MAGIC {
        Err(anyhow!("Bad magic number: {:x?}", magic_buf))?;
    }
    macro_rules! send_finish_response {
        () => {
            socket.write_u8(Message::Finish as u8)?;
        };
    }
    send_finish_response!();

    let list_file_length = socket.read_u32::<LE>()?;
    let mut buf = Cursor::new(Vec::new());
    io::copy(
        &mut Read::by_ref(&mut socket).take(list_file_length as u64),
        &mut buf,
    )?;
    let entries: Vec<Entry> = bincode_deserialize_compress(&mut buf.into_inner().as_slice())?;
    send_finish_response!();

    let send_configs_length = socket.read_u32::<LE>()?;
    let mut buf = Cursor::new(Vec::new());
    io::copy(
        &mut Read::by_ref(&mut socket).take(send_configs_length as u64),
        &mut buf,
    )?;
    let send_configs: SendConfigs = bincode_deserialize_compress(&mut buf.into_inner().as_slice())?;
    send_finish_response!();

    let sync_dest_dir = if let Some(b) = send_configs.src_basename {
        dest_dir.as_ref().join(b)
    } else {
        dest_dir.as_ref().to_path_buf()
    };

    let send_list = generate_send_list(&entries, &sync_dest_dir)?;
    let mut buf = Cursor::new(Vec::new());
    bincode_serialize_compress(&mut buf, &send_list)?;
    socket.write_u32::<LE>(buf.get_ref().len() as u32)?;
    io::copy(&mut buf.into_inner().as_slice(), &mut socket)?;

    let message_u8 = socket.read_u8()?;
    if message_u8 != Message::Finish as u8 {
        Err(anyhow!("Unexpected message: {}", message_u8))?;
    }

    receive(&mut socket, &sync_dest_dir)?;

    send_finish_response!();

    drop(socket);

    println!("Done!");

    Ok(())
}
