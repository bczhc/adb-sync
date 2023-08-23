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
- Run ./build-rust

## Script implementation

For the initial script implementation, see [`script-impl`](https://github.com/bczhc/adb-sync/tree/script-impl) branch.
