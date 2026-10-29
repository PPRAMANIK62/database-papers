# Database papers

My notes from learning how databases work: the papers, specs and design docs I read, and what I learned from each.

Alongside, I'm building [onyxdb](https://github.com/PPRAMANIK62/onyxdb), a SQL database in Rust.

## Now reading

3. `bufpool`: how a database keeps pages in memory, and when it writes them back.

## Done

| # | Topic | Sources |
|---|-------|---------|
| 1 | [vfs](1-vfs): how a database talks to files, and what a crash does to them | SQLite VFS docs, Dan Luu, OSDI '14, ATC '20 |
| 2 | [pagefile](2-pagefile): how a database stores data in fixed-size pages | CMU 15-445, SQLite file format, PostgreSQL page layout, Ross Williams on CRCs |
