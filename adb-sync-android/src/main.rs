#![feature(try_blocks)]

/// This is the main executive binary, and is aggregated (multi-call).
use anyhow::anyhow;
use std::env;
use std::ffi::OsString;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let binary_name: Option<OsString> = try {
        let self_path = env::args().next()?;
        Path::new(&self_path).file_name()?.to_os_string()
    };
    return match binary_name {
        None => Err(anyhow!("Cannot get binary name")),
        Some(name) => match name.to_str() {
            Some(adb_sync::ANDROID_CALL_NAME_IP_CHECKER) => adb_sync_android::ip_checker::main(),
            Some(adb_sync::ANDROID_CALL_NAME_TCP_SERVER) => adb_sync_android::tcp_server::main(),
            Some(adb_sync::ANDROID_CALL_NAME_STDIO_SERVER) => {
                adb_sync_android::stdio_server::main()
            }
            Some(other) => Err(anyhow!("Invalid binary name: {}", other)),
            None => Err(anyhow!("Invalid binary name")),
        },
    };
}
