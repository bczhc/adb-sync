adb-sync
---

Usage: `adb-sync <android-src-dir> <dest-dir>`

Pull directories from Android incrementally

## Limitations and Notes
- Filenames shouldn't contain newlines
- Needs to install some utilities in Termux on Android
- Filesystems should record file modifications times
- No multiple files/directories and file exclusion support

I've found project https://github.com/google/adb-sync and https://github.com/jb2170/better-adb-sync,
but their sync speed is quite slow; can't fulfill my personal requirements :).
