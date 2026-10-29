# 2. pagefile

How a database stores data in fixed-size pages: a header, checksums, slotted pages, a freelist and overflow chains.

My implementation: [`pagefile` in onyxdb](https://github.com/PPRAMANIK62/onyxdb/tree/main/crates/pagefile).

## References

1. [CMU 15-445 notes: Database Storage I](https://15445.courses.cs.cmu.edu/spring2026/notes/03-storage1.pdf) and [II](https://15445.courses.cs.cmu.edu/spring2026/notes/05-storage2.pdf): files, pages, slotted pages, tuple layout and large values, then the other ways to store tuples, as taught in class.
2. [SQLite Database File Format](https://www.sqlite.org/fileformat2.html) (SQLite docs): a real, fully specified format with a header, page types, cell pointers, freeblocks, overflow chains and a freelist.
3. [PostgreSQL Database Page Layout](https://www.postgresql.org/docs/current/storage-page-layout.html) (PostgreSQL docs): the same problem solved with line pointers and a page checksum, to compare against SQLite's choices.
4. [A Painless Guide to CRC Error Detection Algorithms](http://www.ross.net/crc/download/crc_v3.txt) (Ross Williams): how a CRC works, from polynomial division up to a table-driven version.

## What I learned

- Databases split a file into fixed-size pages and always read and write whole pages. SQLite defaults to 4 KiB, PostgreSQL uses 8 KiB, InnoDB 16 KiB.
- A slotted page keeps slots at the front and items at the back, growing toward each other. Items can move inside the page while their slot numbers stay the same.
- That stability matters: PostgreSQL indexes point at rows by (page, slot), so a row's slot can never change.
- SQLite reuses deleted space through a chain of freeblocks, and tracks free pages in trunk pages that list them, so freeing a page usually touches only the trunk.
- A value too big for a page goes in a chain of overflow pages, each pointing to the next.
- A CRC is long division in binary, with XOR as subtraction. The table-driven version precomputes every byte value, so each byte costs one lookup, one shift and one XOR.
- Every CRC standard publishes a check value, the CRC of `123456789`. It's the quickest test that an implementation is right.
- A checksum catches flipped bits and torn writes, but not a lost write: the old page still has a valid checksum.
