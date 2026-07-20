# Architectural Decisions

## ADR-001: Manual arg parsing over clap
**Context:** Network unavailable during Cycle 1, preventing crate downloads.
**Decision:** Use `std::env::args()` with manual `--help` handling instead of `clap`.

## ADR-002: Trait-based HTTP abstraction for testability
**Context:** Cycle 2 needed weather API fetching while keeping tests deterministic.
**Decision:** Define `HttpClient` trait. `RealHttpClient` uses reqwest blocking. `MockHttpClient` returns canned responses.

## ADR-003: Typed error enum with exit codes
**Context:** Cycle 3 needed structured error handling for scripts and user clarity.
**Decision:** `WeatherError` enum with 5 variants (Network, Timeout, CityNotFound, InvalidResponse, Internal). Exit codes: 0 success, 1 usage/not-found, 2 external failure.
