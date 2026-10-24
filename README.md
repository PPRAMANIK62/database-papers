# Database papers

I'm learning how databases work by reading the papers, specs and design docs behind them and building the core idea of each one myself.

The reading list follows the structure of Alex Petrov's *Database Internals*: first how one machine stores data (file formats, pages, B-trees, log-structured storage), then how many machines agree on it, then real systems and how they break.

For each paper the loop is the same:

1. Read it and understand what it explains.
2. Build the main idea in Rust, from scratch, with no dependencies where I can manage it.
3. Check the build against the real system (for example, my SQLite parser has to agree with `sqlite3`).
4. Write about it.

Each paper gets its own folder, numbered in the order I take them: `N-<slug>/`, with my implementation inside it.

## Now working on

SQLite Database File Format: [`dbparse`](1-sqlite-file-format/dbparse), a parser for SQLite's `.db` files.

## Done

Papers I've finished, with what I built for each. A row goes in when both the reading and the implementation are done.

| # | Paper | Implementation | Notes |
|---|-------|----------------|-------|
