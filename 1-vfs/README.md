# 1. vfs

How a database talks to files, and what a crash does to them. Every file operation goes through one interface. A file system in memory can lose power, tear a write or fail an `fsync`. A harness crashes code after every change it makes.

My implementation: [`vfs` in onyxdb](https://github.com/PPRAMANIK62/onyxdb/tree/main/crates/vfs).

## References

1. [The SQLite OS Interface or "VFS"](https://www.sqlite.org/vfs.html) (SQLite docs): how a real database puts every file operation behind one interface, and why.
2. [Files are hard](https://danluu.com/file-consistency/) (Dan Luu): why "just write to a file" is not safe.
3. [All File Systems Are Not Created Equal](https://www.usenix.org/system/files/conference/osdi14/osdi14-paper-pillai.pdf) (OSDI '14): real databases and version-control tools that lose data when the power fails, and why.
4. [Can Applications Recover from fsync Failures?](https://www.usenix.org/system/files/atc20-rebello.pdf) (ATC '20): what ext4, XFS and Btrfs do when `fsync` fails, and how none of PostgreSQL, LMDB, LevelDB, SQLite or Redis handles it safely in every case.

## What I learned

- When `write` returns, the bytes are only in the kernel's memory. They're on the disk once `fsync` returns.
- A write isn't atomic. A crash can keep all of it, none of it, or part of it.
- A file's name lives in its directory, not in the file. Creating, renaming or removing a file needs an `fsync` on the directory too, or a crash can undo it.
- The safe way to replace a file: write a temporary file, `fsync` it, rename it over the old one, then `fsync` the directory.
- File systems differ in what a crash keeps, and real programs quietly depend on behaviour nobody promised. LevelDB didn't sync the directory entries of its table files, and ZooKeeper didn't for its logs, so a crash could make those files vanish.
- On Linux, a failed `fsync` marks the unwritten pages clean. The new bytes are still readable from memory, but they never reach the disk, and the next `fsync` succeeds. So retrying a failed `fsync` loses data. PostgreSQL retried, and found this out in 2018.
- In 2020, none of PostgreSQL, LMDB, LevelDB, SQLite or Redis handled a failed `fsync` safely in every case.
- SQLite sends every file operation through one interface, its VFS. The rest of SQLite doesn't know which operating system or storage is underneath.
- That same interface makes crash testing possible. A file system in memory that remembers what isn't synced yet can crash after any change, and a seed makes each crash repeatable.
