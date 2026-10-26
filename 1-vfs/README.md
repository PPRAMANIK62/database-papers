# 1. vfs

How a database talks to files, and what a crash does to them. Every file operation goes through one interface. A file system in memory can lose power, tear a write or fail an `fsync`. A harness crashes code after every change it makes.

My implementation: [`vfs` in onyxdb](https://github.com/PPRAMANIK62/onyxdb/tree/main/crates/vfs).

## References

1. [The SQLite OS Interface or "VFS"](https://www.sqlite.org/vfs.html) (SQLite docs): how a real database puts every file operation behind one interface, and why.
2. [Files are hard](https://danluu.com/file-consistency/) (Dan Luu): why "just write to a file" is not safe.
3. [All File Systems Are Not Created Equal](https://www.usenix.org/system/files/conference/osdi14/osdi14-paper-pillai.pdf) (OSDI '14): real databases and version-control tools that lose data when the power fails, and why.
4. [Can Applications Recover from fsync Failures?](https://www.usenix.org/system/files/atc20-rebello.pdf) (ATC '20): what ext4, XFS and Btrfs do when `fsync` fails, and how none of PostgreSQL, LMDB, LevelDB, SQLite or Redis handles it safely in every case.

## What I learned
