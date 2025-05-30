#![feature(try_blocks)]
#![feature(yeet_expr)]

use crate::stream::protocol::SendConfig;
use crate::unix_path::UnixPath;
use bincode::config::Configuration;
use bincode::{Decode, Encode};
use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};
use once_cell::sync::Lazy;
use std::env::{args, current_exe};
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};

pub mod crc;
mod send_stream;
pub mod stream;
pub mod unix_path;

pub const ADB_SYNC_PORT: u16 = 5001;
pub const IP_CHECKER_PORT: u16 = 5002;
pub static ANY_IPV4_ADDR: Lazy<Ipv4Addr> = Lazy::new(|| "0.0.0.0".parse().unwrap());

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
pub struct Config {
    pub send_config: SendConfig,
    pub dest_path: PathBuf,
    pub ignore_mtime: bool,
}

macro_rules! count {
    () => (0_usize);
    ($x:expr) => (1_usize);
    ( $x:expr, $($xs:expr),* ) => (1usize + count!($($xs),*));
}

macro_rules! const_android_call_name {
    ($($name:tt),+ $(,)?) => {
        $(
            paste::paste! {
                pub const [<ANDROID_CALL_NAME_ $name:upper>]: &str = $name;
            }
        )*
        pub static ANDROID_CALL_NAMES: [&str; count![$($name),*]] = [$($name),*];
    };
}

const_android_call_name!("ip-checker", "tcp-server", "stdio-server", "get-ip");

pub fn any_ipv4_socket(port: u16) -> SocketAddr {
    (*ANY_IPV4_ADDR, port).into()
}

#[derive(Encode, Decode, Debug)]
pub struct Entry {
    pub path: UnixPath,
    pub size: u64,
    pub modified: SystemTime,
}

pub fn cli_args() -> Vec<String> {
    args().skip(1).collect::<Vec<_>>()
}

pub fn bincode_config() -> Configuration {
    bincode::config::standard().with_variable_int_encoding()
}

pub fn enable_backtrace() {
    // SAFETY: in a single-threaded case
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }
}

pub trait TryReadExact {
    /// Read exact data
    ///
    /// This function blocks. It reads exact data, and returns bytes it reads. The return value
    /// will always be the buffer size until it reaches EOF.
    ///
    /// When reaching EOF, the return value will be less than the size of the given buffer,
    /// or just zero.
    ///
    /// This simulates C function `fread`.
    fn try_read_exact(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

impl<R> TryReadExact for R
where
    R: Read,
{
    fn try_read_exact(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read = 0_usize;
        loop {
            let result = self.read(&mut buf[read..]);
            match result {
                Ok(r) => {
                    if r == 0 {
                        return Ok(read);
                    }
                    read += r;
                    if read == buf.len() {
                        return Ok(read);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub fn index_dir<P: AsRef<Path>>(dir: P, skip_failed: bool) -> io::Result<Vec<Entry>> {
    let walk_dir = jwalk::WalkDir::new(dir.as_ref()).skip_hidden(false);
    let mut entries = Vec::new();
    for x in walk_dir {
        let Ok(entry) = x else {
            if skip_failed {
                eprintln!("Failed to index: {:?}", x);
                continue;
            } else {
                return Err(io::Error::from(x.err().unwrap()));
            }
        };
        if entry.file_type.is_dir() {
            // don't send directories
            continue;
        }
        let result: io::Result<Entry> = try {
            let metadata = entry.metadata()?;
            let path = entry.path();
            let relative_path = pathdiff::diff_paths(&path, dir.as_ref()).unwrap();
            Entry {
                path: relative_path.into(),
                size: metadata.len(),
                modified: metadata.modified()?,
            }
        };
        match result {
            Ok(e) => {
                entries.push(e);
            }
            Err(e) => {
                eprintln!("Error: {:?}", (e, entry));
            }
        }
    }
    Ok(entries)
}

pub fn generate_send_list<P: AsRef<Path>>(
    entries: Vec<Entry>,
    dest_dir: P,
) -> io::Result<Vec<Entry>> {
    let ignore_mtime = mutex_lock!(CONFIG).as_ref().unwrap().ignore_mtime;

    let mut send_list = Vec::new();
    for e in entries {
        let path = &e.path.0;
        let dest_file = dest_dir.as_ref().join(path);
        let send: io::Result<bool> = (|| {
            if !dest_file.exists() {
                return Ok(true);
            }

            let metadata = dest_file.symlink_metadata()?;
            if metadata.len() != e.size {
                return Ok(true);
            }

            if !ignore_mtime && metadata.modified()? != e.modified {
                return Ok(true);
            }

            Ok(false)
        })();
        if send? {
            send_list.push(e)
        }
    }
    Ok(send_list)
}

pub static ANDROID_TMP_DIR: Lazy<&Path> = Lazy::new(|| Path::new("/data/local/tmp"));
pub static ANDROID_ADB_SYNC_TMP_DIR: Lazy<&Path> =
    Lazy::new(|| Path::new("/data/local/tmp/adb-sync"));
pub const ADB_EXE_NAME: &str = "adb";

pub fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[macro_export]
macro_rules! assert_utf8_path {
    ($path:expr) => {
        $path.to_str().expect("Non-UTF8 self_path")
    };
}

pub fn adb_command(subcommand: &str, args: &[&str]) -> io::Result<()> {
    let status = Command::new(ADB_EXE_NAME)
        .arg(subcommand)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(io::Error::other("Failed adb execution"));
    }
    Ok(())
}

pub fn adb_shell<S: AsRef<str>>(shell: S) -> io::Result<()> {
    adb_command("shell", &[shell.as_ref()])
}

pub fn adb_shell_run(binary_name: &str, args: &[&str]) -> io::Result<()> {
    let bin_path = ANDROID_ADB_SYNC_TMP_DIR.join(binary_name);
    let mut joined_args = vec![assert_utf8_path!(bin_path)];
    for &x in args {
        joined_args.push(x);
    }
    adb_command("shell", &joined_args)
}

pub fn android_mktemp() -> io::Result<PathBuf> {
    let timestamp = timestamp_ms();
    adb_shell(format!(
        "mkdir {0} 2>/dev/null || true && touch {0}/{1}",
        ANDROID_ADB_SYNC_TMP_DIR.display(),
        timestamp
    ))?;
    Ok(ANDROID_TMP_DIR
        .join("adb-sync")
        .join(format!("{}", timestamp)))
}

pub fn configure_log() -> anyhow::Result<()> {
    let colors = ColoredLevelConfig::new()
        // use builder methods
        .info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                format!("{}", humantime::format_rfc3339(SystemTime::now())).yellow(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(io::stderr())
        .apply()?;
    Ok(())
}

pub fn self_dirname() -> PathBuf {
    let mut buf = current_exe()
        .expect("Can't get path of the current executable")
        .canonicalize()
        .expect("Can't canonicalize path");
    assert!(buf.pop());
    buf
}

#[macro_export]
macro_rules! mutex_lock {
    ($e:expr) => {
        $e.lock().unwrap()
    };
}
