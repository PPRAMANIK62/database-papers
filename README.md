# Database papers

I'm learning how databases work by reading the papers, specs and design docs behind them, then designing and building each piece myself in Rust.

The reading list is organized around builds. Each build is one Rust crate on one topic (a file of pages, a buffer pool, a B-tree, a write-ahead log, an LSM tree, Raft, a graph store and so on), and each one draws on several sources instead of translating any single one. It goes from how one machine stores data, to how many machines agree on it, to graph databases on object storage and async runtimes.

For each build the loop is the same:

1. Read everything on the topic and understand it.
2. Design my own version, then build it in Rust with no dependencies except my own earlier builds.
3. Check it against a simple model, and against a real system wherever one does the same job.
4. Write about it.

Each build gets its own folder, numbered in the order of the reading list: `N-<crate>/`, with my implementation inside it. Later builds use earlier ones as dependencies.

## Now working on

Build 1, `pagefile`: a file of fixed-size, checksummed pages, with slotted pages, a freelist and overflow chains.

Before switching to builds I started with one spec, the SQLite file format, and wrote [`dbparse`](https://github.com/PPRAMANIK62/database-papers/tree/3be8a6e/1-sqlite-file-format/dbparse), a parser for SQLite's `.db` files.

## Done

Builds I've finished. A row goes in when both the reading and the implementation are done.

| # | Build | What it is | Sources |
|---|-------|------------|---------|
