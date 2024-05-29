adb-sync
---

## Usage

> TODO

The supported Android version is Android 7.0 Nougat (API Version 24) and above, because it depends on
`ifaddrs` which is supported officially above this version.

## Build

- Set up Rust with NDK toolchain

  An example configuration file:
  ```toml
  # <project>/.cargo/config
  
  [build]
  #target = "aarch64-linux-android"
  
  [target.aarch64-linux-android]
  linker = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"
  ar = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  
  [env]
  TARGET_CC = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"
  TARGET_AR = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  ```

- Install Android targets using `rustup`:

  ```shell
  rustup target add aarch64-linux-android
  ```
  If your Android architecture is not aarch64, choose
  some others (also change `$android_target` in `./build-rust` correspondingly):
    - aarch64-linux-android
    - armv7-linux-androideabi
    - i686-linux-android
    - x86_64-linux-android
- Run `./build-rust`

## Limitations and Notes

- Relies on `mtime`s
- No multiple files/directories and file exclusion support
- Empty directories won't be synced
- Only supports regular files (that's, totally ignores symlink, pipe etc.; no reflink or hard link awareness)

I've found project https://github.com/google/adb-sync
and https://github.com/jb2170/better-adb-sync, but their
sync speed is quite slow; can't fulfill my personal
requirements :).

## Script implementation

For the initial script implementation, see [`script-impl`](https://github.com/bczhc/adb-sync/tree/script-impl) branch.

*For syncing over network, see https://github.com/bczhc/FileSync*
