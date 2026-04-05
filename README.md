# Prototypes

A personal monorepo for small experiments across different concepts and languages.

## Purpose

- Keep all prototype projects in one place
- Learn by building focused, self-contained exercises
- Compare approaches across languages and stacks

## Layout

| Theme | What lives here |
|-------|-----------------|
| **`storage/`** | Local engine pieces: KV stores, iterators, persistence, WAL, B-trees, LSM, compaction, query parsing |
| **`consensus/`** | Distributed agreement (Raft, etc.) |
| **`vectors/`** | Vector indexes (flat, HNSW, …) |

Placeholders under `storage/` and `vectors/` use `.gitkeep` until you add a `Cargo.toml` and code.

## Workspace

Rust crates are members of the root [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). From the repo root:

```bash
cargo test --workspace
cargo run -p kv_store
cargo run -p typed_kv
cargo run -p iterator_scan
cargo test -p raft_simple
```

Or run inside a crate directory as before (Cargo discovers the workspace root).

## Crates

| Path | Description | Run | Test |
|------|-------------|-----|------|
| `storage/01_kv_store/` | In-memory key-value store | `cargo run -p kv_store` | `cargo test -p kv_store` |
| `storage/02_typed_kv/` | Typed key-value store (`Value` enum) | `cargo run -p typed_kv` | `cargo test -p typed_kv` |
| `storage/03_iterator_scan/` | Iterator / scan exercise | `cargo run -p iterator_scan` | `cargo test -p iterator_scan` |
| `consensus/raft_simple/` | Minimal Raft scaffold (see `problem.md`) | — | `cargo test -p raft_simple` |
