# DESIGN.md — Zero-Copy Log Analyzer

## Overview

This document explains the design decisions made in building the log analyzer, including how the file is read, how parsing avoids unnecessary allocations, where allocations are unavoidable, and how the solution behaves with very large files.

---

## How the File is Read and Processed

The file is opened as a standard `File` and immediately wrapped in a `BufReader<File>`.

`BufReader` maintains an internal read buffer (8KB by default). When we call `.lines()` on it, it reads chunks from disk into that buffer and yields one line at a time as a `Result<String, io::Error>`. At no point is the entire file loaded into memory — only the current buffer chunk is live at any time.

This means memory usage is effectively constant regardless of file size. A 10MB file and a 10GB file consume the same memory during processing.

The `.lines()` iterator handles `\n` and `\r\n` line endings correctly, so we don't need to strip them manually.

---

## How Parsing is Performed Without Unnecessary Allocations

The core parsing logic is in `src/parser.rs`. The key design is using `str::splitn(4, '|')` on the borrowed `&str` line.

`splitn` returns an iterator of `&str` slices — these are pointers into the original string's memory, not new allocations. We extract the second field (the log level) this way and match it directly against string literals. No new `String` is constructed during parsing.

```
line (owned String from BufReader)
 └── level_str: &str  ← just a pointer into `line`, no copy
      └── match "INFO" / "WARN" / "ERROR" ← zero-copy comparison
```

The match arm then increments a `u64` counter. The entire parse path allocates nothing except the line itself (from BufReader).

---

## Where Allocations Are Unavoidable

1. **The line `String`** — `BufReader::lines()` returns `Result<String, _>`. This allocation per line is unavoidable when using the safe `.lines()` API. An alternative would be `read_line(&mut buf)` with a reused `String` buffer, which would reduce per-line allocations to zero by clearing and reusing the same buffer. I chose `.lines()` for clarity and simplicity — for a production system processing billions of lines, `read_line` reuse would be worth the added complexity.

2. **`splitn` result collection** — `splitn` itself is lazy and doesn't allocate, but calling `.next()` on the iterator yields `&str` slices that live as long as the line. This is fine.

---

## Performance Trade-offs Made

- **`.lines()` vs `read_line` reuse**: Chose `.lines()` for readability. Each line incurs one `String` allocation. For most workloads this is fast enough. If profiling showed this as a bottleneck, switching to a reused buffer is a straightforward refactor.

- **`splitn(4, '|')`**: We only split up to 4 parts, so a message field containing `|` characters doesn't cause extra splits or parsing errors. This is intentional.

- **No regex**: Regex would add a dependency and runtime overhead for a simple fixed-format parser. String slicing is faster and more predictable here.

- **No parallelism**: Parallel chunk processing (e.g. with `rayon`) would complicate the streaming model and require either chunked file reading with offset tracking or pre-splitting the file. For most log files, I/O throughput is the bottleneck, not CPU, so parallelism wouldn't help significantly without also parallelizing disk reads. Left as a future improvement.

---

## How the Solution Behaves With Very Large Files

- **Memory**: Constant. Only one `BufReader` buffer (~8KB) + one `Stats` struct (32 bytes) is live at a time.
- **CPU**: Linear in file size. Each line is parsed once with O(1) work.
- **Error resilience**: Malformed lines are counted and skipped — a single corrupt line does not stop processing or crash the program. I/O errors (e.g. disk failure mid-read) propagate up and exit cleanly with an error message.
- **Large line handling**: If a single log line is very long (e.g. a huge message field), `BufReader` will grow its buffer to accommodate it. This is handled by the standard library transparently.

---

## File Structure

```
src/
  main.rs     — CLI argument parsing, file opening, drives the analyze() loop
  parser.rs   — parse_line() function, LogLevel enum, unit tests
  stats.rs    — Stats struct, counters, print_summary()
DESIGN.md     — this file
Cargo.toml    — project manifest, no external dependencies
```

---

## Running the Program

```bash
# Build
cargo build --release

# Run
./target/release/ddn-rust path/to/logfile.log

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Check for clippy warnings
cargo clippy
```