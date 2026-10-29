# 3. bufpool

How a database keeps pages in memory: frames, pins, the write-ahead rule, eviction policies and latches.

My implementation: [`bufpool` in onyxdb](https://github.com/PPRAMANIK62/onyxdb/tree/main/crates/bufpool).

## References

1. [CMU 15-445 notes: Buffer Pools](https://15445.courses.cs.cmu.edu/fall2024/notes/06-bufferpool.pdf) and [Hash Tables](https://15445.courses.cs.cmu.edu/fall2024/notes/07-hashtables.pdf): frames, the page table, pinning, replacement policies, locks against latches, and the hash table behind the page table, as taught in class.
2. [The Internals of PostgreSQL, ch. 8: Buffer Manager](https://www.interdb.jp/pg/pgsql08.html) (Hironobu Suzuki): a real buffer pool, with buffer descriptors, a partitioned buffer table, pinning, clock-sweep eviction and ring buffers.
3. [Are You Sure You Want to Use MMAP in Your DBMS?](https://db.cs.cmu.edu/papers/2022/cidr2022-p13-crotty.pdf) (Crotty, Leis and Pavlo, CIDR '22): why databases don't let the operating system manage their pages.
4. [The LRU-K Page Replacement Algorithm for Database Disk Buffering](https://www.cs.cmu.edu/~christos/courses/721-resources/p297-o_neil.pdf) (O'Neil, O'Neil and Weikum, SIGMOD '93): evict by the time of the K-th most recent reference, so one scan can't flush the pool.
