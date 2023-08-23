adb-sync
---

## Usage

> adb-sync \<adb-src-dir\> \<dest-dir\>

## Build

- Set up Rust with NDK toolchain

  An example configuration file:
  ```toml
  [build]
  #target = "aarch64-linux-android"
  
  [target.aarch64-linux-android]
  linker = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"
  ar = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  
  [env]
  TARGET_CC = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"
  TARGET_AR = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  ```

- Install Android targets using `rustup`
    - aarch64-linux-android
    - armv7-linux-androideabi
    - i686-linux-android
    - x86_64-linux-android
- Run `./build-rust`

## Limitations and Notes
- Relies on mtimes
- No multiple files/directories and file exclusion support
- Empty directories won't be synced
- Only supports regular files (that's, totally ignores symlink, link, reflink, pipe things etc...)

I've found project https://github.com/google/adb-sync
and https://github.com/jb2170/better-adb-sync, but their
sync speed is quite slow; can't fulfill my personal
requirements :).

## Script implementation

For the initial script implementation, see [`script-impl`](https://github.com/bczhc/adb-sync/tree/script-impl) branch.
