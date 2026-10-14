# rng

A small command-line random number generator.

Integers are drawn from the inclusive range `min..=max` (`i64`). Floats are drawn from the half-open range `[min, max)` (`f64`). Each number is printed on its own line. Pass `--seed` when the sequence must be repeatable.

## Build

```bash
cargo build
cargo build --release
```

The debug binary is `target/debug/rng`. The release binary is `target/release/rng`.

## Usage

```text
rng [-n COUNT] [-m MIN] [-M MAX] [-f] [-s SEED]
```

| Flag | Meaning | Default |
| --- | --- | --- |
| `-n`, `--count` | How many numbers to print. Must be at least 1. | `1` |
| `-m`, `--min` | Lower bound (inclusive). | `1` |
| `-M`, `--max` | Upper bound. Inclusive for integers, exclusive for floats. | `100` |
| `-f`, `--float` | Sample `f64` values from `[min, max)`. | off (integers) |
| `-s`, `--seed` | `u64` seed for reproducible output. | unset (OS entropy) |

Integer bounds must be written as integers in the `i64` range, including the full domain from `-9223372036854775808` to `9223372036854775807`. Negative bounds are accepted (`-m -5 -M -1`). A fractional token such as `1.5` is rejected in integer mode. Float mode rejects an empty half-open interval (`min >= max`), and integer mode rejects an inverted range (`min > max`).

## Examples

Default: one integer from 1 through 100.

```bash
cargo run
```

Five integers from 10 through 20:

```bash
cargo run -- -n 5 -m 10 -M 20
```

Three floats from the half-open interval `[0, 1)`:

```bash
cargo run -- -f -m 0 -M 1 -n 3
```

The same five integers every time the seed is 42:

```bash
cargo run -- -s 42 -n 5
```

```text
53
55
64
41
4
```

Seeded floats from `[0, 1)`:

```bash
cargo run -- -f -s 42 -n 3 -m 0 -M 1
```

```text
0.5265574090027738
0.5427252099031439
0.6364650991438949
```

## Test

```bash
cargo test
```

The unit tests cover default arguments, inverted integer ranges, fractional integer bounds, empty float ranges, seeded integer and float sequences, and sampling across the full `i64` domain.
