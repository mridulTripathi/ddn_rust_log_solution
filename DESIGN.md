# DESIGN.md — Architecture and Decision Record

## Purpose

This document captures the application architecture, the route selected for implementation, and why those choices were made. Usage and run commands are intentionally documented in `README.md`.

## Chosen Architecture

The app follows a small pipeline-oriented architecture with clear module boundaries:

- `src/main.rs`: CLI entrypoint, format selection, file streaming, chunk orchestration, and final reporting.
- `src/parser.rs`: domain parser for a single log line (`parse_line`) and log format configuration (`LogFormat`).
- `src/stats.rs`: aggregation model (`Stats`) for severity counters and malformed-line breakdown.

### Data flow

1. Open file and stream it line-by-line via `BufReader`.
2. Accumulate lines into bounded chunks.
3. Parse each chunk in parallel (`rayon`) and produce local stats.
4. Merge local stats into a global accumulator.
5. Print final summary (valid counts + malformed details).

## Route Chosen and Why

### 1) Streamed reads instead of loading whole file

**Chosen route:** `BufReader` + iterator-based line reading.

**Why:**

- Keeps memory bounded for large files.
- Naturally handles incremental processing.
- Avoids upfront memory spikes from full-file reads.

### 2) Chunk-level parallel parsing instead of full-file parallel split

**Chosen route:** sequential I/O + parallel parse per chunk.

**Why:**

- Preserves a simple and safe I/O model.
- Gains parallel CPU parsing where helpful.
- Avoids complexity of byte-offset partitioning and newline boundary stitching.

### 3) Configurable delimiters with a single parser path

**Chosen route:** `LogFormat` abstraction (`pipe`, `csv`, `tsv`) and one parsing function.

**Why:**

- Keeps parsing logic centralized and easier to maintain.
- Supports multiple input styles without code duplication.
- Preserves one validation and error-reporting flow across formats.

### 4) Malformed-line reasons instead of only malformed totals

**Chosen route:** return malformed reasons from parser and aggregate by reason.

**Why:**

- Improves operational visibility for data quality issues.
- Makes troubleshooting easier than a single malformed counter.
- Keeps parser responsibilities explicit (classify, do not silently drop context).

## Error Handling Strategy

- File open/read failures are surfaced as hard errors and exit the process.
- Individual malformed lines are treated as non-fatal and included in summary reporting.
- Unknown CLI args and unsupported formats fail fast with clear messages.

## Performance Characteristics

- **Memory:** bounded by `BufReader` internals + current chunk + stats accumulator.
- **CPU:** linear in line count; parsing cost is parallelized per chunk.
- **Scalability behavior:** larger files benefit from streaming; CPU-heavy parsing benefits more from rayon than tiny files.

## Trade-offs and Non-goals

- Chosen for clarity over maximal micro-optimization (`.lines()` allocates a `String` per line).
- Current parser is delimiter-based and intentionally not regex-driven.
- The tool assumes a minimum 4-field schema and known level values.

## Future Extensions

- Add parser unit tests for each format and malformed-reason variant.
- Add dedicated benchmark harness (for example with Criterion) for stable regressions tracking.
- Support custom schemas (field order mapping) if input contracts expand.