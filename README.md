# ddn_rust

`ddn_rust` is a command-line log analyzer written in Rust.

It reads a log file, counts `INFO`/`WARN`/`ERROR` records, and reports malformed lines separately (including malformed reasons). The analyzer supports multiple delimiters through `--format`.

## Supported input format

The analyzer expects at least 4 fields per line:

`timestamp<delimiter>level<delimiter>component<delimiter>message`

Supported delimiters (via `--format`):

- `pipe` (`|`) - default
- `csv` (`,`)
- `tsv` (tab)

Recognized levels:

- `INFO`
- `WARN`
- `ERROR`

## Quick start

Build:

```bash
cargo build --release
```

Run using the included sample:

```bash
cargo run --release -- test.log --format pipe
```

Run with your own log file:

```bash
cargo run --release -- /path/to/your.log --format pipe
```

Examples for other formats:

```bash
cargo run --release -- /path/to/your.csv --format csv
cargo run --release -- /path/to/your.tsv --format tsv
```

## Output

The program prints:

- total `INFO` count
- total `WARN` count
- total `ERROR` count
- total malformed line count
- malformed breakdown by reason (when malformed lines are present)

## Notes

- `test.log` is included in this repository and can be used for quick verification.
- For implementation details and design choices, see `DESIGN.md`.
