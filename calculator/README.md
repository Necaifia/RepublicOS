# Calculator CLI

A command-line calculator built by Pantheon — the autonomous engineering operating system.

## Usage

REPL mode:

```sh
cargo run
> 2 + 3
= 5
> 10 / 0
Error: Division by zero
> quit
```

## Library

```rust
use calculator::{Calculator, CalcOp, calculate};

let calc = Calculator::new();
assert_eq!(calc.eval("2 + 3").unwrap(), 5.0);

assert_eq!(calculate(CalcOp::Multiply, 4.0, 5.0).unwrap(), 20.0);
```

## Tests

```sh
cargo test
```

17 tests covering arithmetic, error handling, edge cases.
