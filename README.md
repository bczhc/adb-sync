adb-sync
---

## Usage

Note: The supported Android version is Android 7.0 Nougat (API Version 24) and above, because it depends on
`ifaddrs` which is supported officially only above this version.

<pre><u style="text-decoration-style:solid"><b>Usage:</b></u> <b>adb-sync</b> [OPTIONS] &lt;ANDROID_DIR&gt; &lt;HOST_DIR&gt;

<u style="text-decoration-style:solid"><b>Arguments:</b></u>
  &lt;ANDROID_DIR&gt;
          Path of the source directory

  &lt;HOST_DIR&gt;
          Path of the destination directory

<u style="text-decoration-style:solid"><b>Options:</b></u>
      <b>--android-bin-search-path</b> &lt;ANDROID_BIN_SEARCH_PATH&gt;
          Search `adb-sync-android` in this path. Default to where `adb-sync` locates
          
          [default: .]

      <b>--no-stdio</b>
          Do not fall back to the stdio method when Android IP is unavailable

      <b>--no-tcp</b>
          Use stdio only

      <b>--skip-failed</b>
          Skip indexing failure

      <b>--android-ip</b> &lt;ANDROID_IP&gt;
          Manually specify the Android IP instead of automatic detection.
          
          Only used in TCP mode.

  <b>-h</b>, <b>--help</b>
          Print help (see a summary with &apos;-h&apos;)</pre>

## Build

- Set up Rust with NDK toolchain

  An example configuration file (replace these paths with yours):
  ```toml
  # <project>/.cargo/config
  
  [build]
  [target.aarch64-linux-android]
  linker = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"
  ar = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  
  [env]
  TARGET_CC = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"
  TARGET_AR = "/home/bczhc/bin/AndroidSdk/ndk-ln/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  ```

- Install Android targets using `rustup`:

  ```shell
  rustup target add aarch64-linux-android x86_64-unknown-linux-musl
  ```
  If your Android architecture is not aarch64, choose
  some others (also change `$android_target` in `./build-rust` correspondingly):

    - aarch64-linux-android
    - armv7-linux-androideabi
    - i686-linux-android
    - x86_64-linux-android
- Run `./build-rust`

### Run

After `./build-rust`, run:

```bash
./adb-sync
```

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
