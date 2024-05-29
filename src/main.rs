#![feature(try_blocks)]

use std::ffi::OsStr;
use std::fs::create_dir_all;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread::{sleep, spawn};
use std::time::Duration;

use anyhow::anyhow;
use clap::Parser;
use colored::Colorize;
use log::{debug, info};
use once_cell::sync::Lazy;
use readwrite::ReadWrite;

use adb_sync::stream::host::start;
use adb_sync::stream::protocol::SendConfig;
use adb_sync::stream::ReadWriteFlush;
use adb_sync::{
    adb_command, adb_shell, adb_shell_run, android_mktemp, assert_utf8_path, configure_log,
    mutex_lock, ADB_EXE_NAME, ADB_SYNC_PORT, ANDROID_ADB_SYNC_TMP_DIR, ANDROID_CALL_NAMES,
    ANDROID_CALL_NAME_GET_IP, ANDROID_CALL_NAME_IP_CHECKER, ANDROID_CALL_NAME_STDIO_SERVER,
    ANDROID_CALL_NAME_TCP_SERVER, IP_CHECKER_PORT,
};

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

pub struct Config {
    send_config: SendConfig,
    dest_path: PathBuf,
}

const ANDROID_BIN_NAME: &str = "adb-sync-android";

#[derive(clap::Parser)]
pub struct Args {
    /// Path of the source directory
    pub android_dir: PathBuf,
    /// Path of the destination directory
    pub host_dir: PathBuf,
    #[arg(default_value = clap_leaked_self_dirname(), long, alias = "absp")]
    pub android_bin_search_path: PathBuf,
    /// Do not fall back to the stdio method when Android IP is unavailable.
    #[arg(conflicts_with = "no_tcp", long, alias = "ns", default_value = "false")]
    pub no_stdio: bool,
    /// Use stdio only.
    #[arg(
        conflicts_with = "no_stdio",
        long,
        alias = "nt",
        default_value = "false"
    )]
    pub no_tcp: bool,
}

pub fn clap_leaked_self_dirname() -> &'static OsStr {
    Box::leak(
        adb_sync::self_dirname()
            .into_os_string()
            .into_boxed_os_str(),
    )
}

pub fn main() -> anyhow::Result<()> {
    configure_log()?;
    let args = Args::parse();

    // Like rsync, if the source path ends with a slash, put all the received files
    // under a directory with the same base name as the source path.
    let real_dest_dir = if format!("{}", args.android_dir.display()).ends_with('/') {
        args.host_dir.clone()
    } else {
        args.host_dir.join(args.android_dir.file_name().unwrap())
    };
    create_dir_all(&real_dest_dir)?;

    info!("Source path: {}", args.android_dir.display());
    info!("Destination path: {}", args.host_dir.display());
    info!("Receive files at: {}", real_dest_dir.display());

    mutex_lock!(CONFIG).replace(Config {
        send_config: SendConfig {
            path: args.android_dir,
        },
        dest_path: real_dest_dir,
    });

    let android_binary = args.android_bin_search_path.join(ANDROID_BIN_NAME);
    if !android_binary.exists() {
        return Err(anyhow!(
            "Android binary doesn't exist: {}",
            android_binary.display()
        ));
    };

    info!("{}", "Preparing Android binaries...".cyan().bold());
    prepare_android_binaries(android_binary)?;

    let android_ip = if args.no_tcp {
        None
    } else {
        get_connectable_ip()?
    };

    match android_ip {
        None => {
            if args.no_stdio {
                return Err(anyhow!(
                    "no_stdio is enabled. Won't fall back to this method."
                ));
            } else {
                info!("Transfer via stdio");
                stdio_transfer()?;
            }
        }
        Some(ip) => {
            info!("Transfer via TCP");
            info!("Use Android IP: {}", ip);
            tcp_transfer(ip)?;
        }
    }

    Ok(())
}

fn tcp_transfer(ip: IpAddr) -> anyhow::Result<()> {
    let android_child = spawn(|| {
        adb_shell_run(ANDROID_CALL_NAME_TCP_SERVER, &[]).unwrap();
    });
    sleep(Duration::from_secs(1));
    let guard = mutex_lock!(CONFIG);
    let config = guard.as_ref().unwrap();

    let tcp_stream = TcpStream::connect(SocketAddr::new(ip, ADB_SYNC_PORT))?;

    start(tcp_stream, config.send_config.clone(), &config.dest_path)?;

    android_child.join().unwrap();
    Ok(())
}

fn stdio_transfer() -> anyhow::Result<()> {
    let guard = mutex_lock!(CONFIG);
    let config = guard.as_ref().unwrap();

    let mut child = Command::new("adb")
        .arg("shell")
        .arg(assert_utf8_path!(
            ANDROID_ADB_SYNC_TMP_DIR.join(ANDROID_CALL_NAME_STDIO_SERVER)
        ))
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let process_stdin = child.stdin.take().unwrap();
    let process_stdout = child.stdout.take().unwrap();
    let stream = ReadWriteFlush(ReadWrite::new(process_stdout, process_stdin));
    start(stream, config.send_config.clone(), &config.dest_path)
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
    sleep(Duration::from_secs(1));
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
