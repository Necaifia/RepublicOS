# Release v1.0.0

**weather-cli** — A CLI tool to fetch and display current weather for any city.

## Changes since init

- **Cycle 1:** Rust project skeleton, CLI arg parsing, --help flag
- **Cycle 2:** wttr.in integration, trait-based HttpClient, mock tests
- **Cycle 3:** WeatherError enum with 5 variants, exit codes (0/1/2), user-friendly messages, --version flag

## Quality

- 13 tests, 0 failures
- 0 compiler warnings
- 0 clippy issues
- Release binary verified

## Usage

```bash
weather-cli London
weather-cli --help
weather-cli --version
```
