# 1. pagefile

How a database stores data in fixed-size pages: a header, checksums, slotted pages, a freelist and overflow chains.

My implementation: [`pagefile` in onyxdb](https://github.com/PPRAMANIK62/onyxdb/tree/main/crates/pagefile).

## References

1. [CMU 15-445 notes: Database Storage I](https://15445.courses.cs.cmu.edu/fall2024/notes/03-storage1.pdf) and [II](https://15445.courses.cs.cmu.edu/fall2024/notes/04-storage2.pdf): files, pages, slotted pages and tuple layout, as taught in class.
2. [SQLite Database File Format](https://www.sqlite.org/fileformat2.html) (SQLite docs): a real, fully specified format with a header, page types, cell pointers, freeblocks, overflow chains and a freelist.
3. [PostgreSQL Database Page Layout](https://www.postgresql.org/docs/current/storage-page-layout.html) (PostgreSQL docs): the same problem solved with line pointers and a page checksum, to compare against SQLite's choices.
4. [A Painless Guide to CRC Error Detection Algorithms](http://www.ross.net/crc/download/crc_v3.txt) (Ross Williams): how a CRC works, from polynomial division up to a table-driven version.

## What I learned
