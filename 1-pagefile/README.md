# 1. pagefile

A file of fixed-size, numbered pages, the bottom layer of the database. Page 0 is a header that describes the file, and every page ends with a CRC-32C checksum. Pages can hold many variable-size items (slotted pages), freed pages are reused through a freelist, and values too big for a page go in overflow chains.

The code, its format and the decisions behind it are in the `pagefile` crate's README in `db`. The buffer pool (build 2) and the B-tree (build 5) are built on it.

## References

1. [CMU 15-445 notes: Database Storage I](https://15445.courses.cs.cmu.edu/fall2024/notes/03-storage1.pdf) and [II](https://15445.courses.cs.cmu.edu/fall2024/notes/04-storage2.pdf): files, pages, slotted pages and tuple layout, as taught in class.
2. [SQLite Database File Format](https://www.sqlite.org/fileformat2.html) (SQLite docs): a real, fully specified format with a header, page types, cell pointers, freeblocks, overflow chains and a freelist.
3. [PostgreSQL Database Page Layout](https://www.postgresql.org/docs/current/storage-page-layout.html) (PostgreSQL docs): the same problem solved with line pointers and a page checksum, to compare against SQLite's choices.
4. [A Painless Guide to CRC Error Detection Algorithms](http://www.ross.net/crc/download/crc_v3.txt) (Ross Williams): how a CRC works, from polynomial division up to a table-driven version.

## What I learned

Notes go here as the build goes on: what surprised me, what I'd do differently, and where my design ended up differing from SQLite's and PostgreSQL's.
