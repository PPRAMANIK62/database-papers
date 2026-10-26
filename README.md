# Database papers

I'm building a SQL database from scratch in Rust, and this repo is how I learn each piece of it. For every layer I read the papers, specs and design docs behind it, then design and build my own version. The database itself lives in its own repo, `db`: one Cargo workspace with a crate per layer. This repo holds the reading and what I learned.

The reading list is organized around builds. Each build is one crate in the database (a file of pages, a buffer pool, a write-ahead log, a B-tree, an LSM tree, transactions, SQL, a server and a client), and each one draws on several sources instead of translating any single one. Once the database works, the list goes on to many machines, graph databases and async runtimes.

For each build the loop is the same:

1. Read everything on the topic and understand it.
2. Design my own version, then build it in Rust with no dependencies except my own earlier crates.
3. Check it against a simple model, and against a real system wherever one does the same job.
4. Write about it.

Each build gets a folder here, numbered in the order of the reading list: `N-<crate>/`, with a README listing the sources it drew on and what I learned from them. The design of each crate, and the decisions behind it, are in that crate's README in `db`.

## Now working on

Build 1, `pagefile`: a file of fixed-size, checksummed pages, with slotted pages, a freelist and overflow chains.

Before switching to builds I started with one spec, the SQLite file format, and wrote [`dbparse`](https://github.com/PPRAMANIK62/database-papers/tree/3be8a6e/1-sqlite-file-format/dbparse), a parser for SQLite's `.db` files.

## Done

Builds I've finished. A row goes in when both the reading and the implementation are done.

| # | Build | What it is | Sources |
|---|-------|------------|---------|
