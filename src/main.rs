use std::path::{Path, PathBuf};

use anyhow::anyhow;
use clap::Parser;
use colored::Colorize;
use log::info;

use adb_sync::{
    adb_command, adb_shell, adb_shell_run, android_mktemp, assert_utf8_path, configure_log,
    ANDROID_ADB_SYNC_TMP_DIR, ANDROID_CALL_NAMES, ANDROID_CALL_NAME_IP_CHECKER,
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
    }

    info!("{}", "Preparing Android binaries...".cyan().bold());
    prepare_android_binaries(android_binary)?;

    adb_shell_run(ANDROID_CALL_NAME_IP_CHECKER, &[])?;

    Ok(())
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
