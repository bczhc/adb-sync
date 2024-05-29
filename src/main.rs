#![feature(try_blocks)]

use std::io;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::os::linux::raw::stat;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::spawn;
use std::time::Duration;

use anyhow::anyhow;
use clap::Parser;
use colored::Colorize;
use log::{debug, info};

use adb_sync::{
    adb_command, adb_shell, adb_shell_run, android_mktemp, assert_utf8_path, configure_log,
    ADB_EXE_NAME, ANDROID_ADB_SYNC_TMP_DIR, ANDROID_CALL_NAMES, ANDROID_CALL_NAME_GET_IP,
    ANDROID_CALL_NAME_IP_CHECKER, IP_CHECKER_PORT,
};

const ANDROID_BIN_NAME: &str = "adb-sync-android";

#[derive(clap::Parser)]
pub struct Args {
    /// Path of the source directory
    pub android_dir: PathBuf,
    /// Path of the destination directory
    pub host_dir: PathBuf,
    #[arg(default_value = ".", long, alias = "absp")]
    pub android_bin_search_path: PathBuf,
}

pub fn main() -> anyhow::Result<()> {
    configure_log()?;
    let args = Args::parse();

    let android_binary = args.android_bin_search_path.join(ANDROID_BIN_NAME);
    if !android_binary.exists() {
        return Err(anyhow!(
            "Android binary doesn't exist: {}",
            android_binary.display()
        ));
    };

    info!("{}", "Preparing Android binaries...".cyan().bold());
    prepare_android_binaries(android_binary)?;

    let android_ip = get_connectable_ip()?;
    println!("{:?}", android_ip);

    Ok(())
}

fn get_connectable_ip() -> anyhow::Result<Option<IpAddr>> {
    let mut child = Command::new(ADB_EXE_NAME)
        .arg("shell")
        .arg(assert_utf8_path!(
            ANDROID_ADB_SYNC_TMP_DIR.join(ANDROID_CALL_NAME_GET_IP)
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().unwrap();
    let mut output = String::new();
    stdout.read_to_string(&mut output)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("Failed adb execution"));
    }

    spawn(|| {
        adb_shell_run(ANDROID_CALL_NAME_IP_CHECKER, &[]).unwrap();
    });

    let ips = output.lines().filter(|x| !x.is_empty()).collect::<Vec<_>>();
    Ok(check_connectivity(&ips))
}

fn check_connectivity(ips: &[&str]) -> Option<IpAddr> {
    let mut handlers = Vec::new();
    for ip in ips {
        let ip_addr = ip.parse().unwrap();
        debug!("check ip: {}", ip_addr);
        let handler = spawn(move || {
            let result: anyhow::Result<IpAddr> = try {
                let mut listener = TcpStream::connect_timeout(
                    &SocketAddr::new(ip_addr, IP_CHECKER_PORT),
                    Duration::from_secs(1),
                )?;
                listener.set_write_timeout(Some(Duration::from_secs(1)))?;
                listener.set_read_timeout(Some(Duration::from_secs(1)))?;
                const ECHO_MESSAGE: [u8; 12] = *b"Please echo\n";
                listener.write_all(&ECHO_MESSAGE)?;
                let mut buf = [0_u8; ECHO_MESSAGE.len()];
                listener.read_exact(&mut buf)?;
                if ECHO_MESSAGE == buf {
                    ip_addr
                } else {
                    Err(anyhow!("Not equal"))?
                }
            };
            result.ok()
        });
        handlers.push(handler);
    }
    for x in handlers {
        if let Some(ip) = x.join().ok().flatten() {
            return Some(ip);
        }
    }
    None
}

pub fn prepare_android_binaries<P: AsRef<Path>>(android_binary: P) -> anyhow::Result<()> {
    info!("{}", "Copying Android binaries...".cyan().bold());
    let android_tmp_binary = android_mktemp()?;
    adb_command(
        "push",
        &[
            assert_utf8_path!(android_binary.as_ref()),
            assert_utf8_path!(android_tmp_binary),
        ],
    )?;
    println!("{:?}", android_tmp_binary);

    info!("{}", "Derive multi-calls via symlinks");
    for name in ANDROID_CALL_NAMES {
        adb_shell(format!(
            "ln -sf {} {}",
            assert_utf8_path!(android_tmp_binary),
            assert_utf8_path!(ANDROID_ADB_SYNC_TMP_DIR.join(name))
        ))?;
    }

    Ok(())
}
