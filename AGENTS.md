# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 Actix Web service that proxies Docker Hub Registry
requests. Application startup and server wiring live in `src/main.rs`.
Routes are grouped under `src/routes/`: `proxy.rs` forwards `/v2/*`,
`auth.rs` forwards Docker Hub token requests, and `ping.rs` and `version.rs`
provide operational endpoints. Shared HTTP client state is in
`src/state/app_state.rs`. `build.rs` injects build and Git metadata. Container
packaging is defined by `Dockerfile`, while `build.sh` builds the image with
Git metadata. There are currently no dedicated test or asset directories.

## Build, Test, and Development Commands

Run the service locally with `cargo run`; it listens on `0.0.0.0:8000`.
Build an optimized binary with `cargo build --release`. Use `cargo check` for
a fast compile check, `cargo fmt --check` to verify formatting, and
`cargo test` to run the test suite (currently there are no repository tests).
Build the container with `docker build -t rammal:latest .`, or run
`./build.sh [tag]` to use the repository's image name and inject Git metadata.
Before changing proxy behavior, verify `/ping`, `/version`, and `/v2/` with
`curl` where network access is available.

## Coding Style & Naming Conventions

Use standard Rust formatting via `cargo fmt` and keep code compatible with
Edition 2024. Follow Rust naming conventions: `snake_case` for functions,
modules, and variables; `PascalCase` for types; and `SCREAMING_SNAKE_CASE` for
constants. Keep route handlers small, return Actix-compatible errors, and
preserve streaming behavior for registry request and response bodies.

## Testing Guidelines

Add unit or integration tests alongside the relevant module, using descriptive
names such as `rewrites_authenticate_header`. At minimum, run `cargo fmt
--check`, `cargo check`, and `cargo test` before submitting changes. Proxy
changes should also include a manual request/response check when practical.

## Commit & Pull Request Guidelines

Existing commits use Conventional Commit-style prefixes such as `feat:`,
`fix:`, `docs:`, `build:`, `ci:`, and `chore:` (for example,
`docs: improve project documentation`). Keep commits focused and describe the
user-visible effect. Pull requests should explain the behavior change, list
validation commands run, and call out Docker or deployment implications. Add
request examples or screenshots only when they clarify an API or operational
change.

## Security & Configuration Tips

The upstream registry and listen address are currently hard-coded. Do not add
credentials or tokens to source, logs, Dockerfiles, or documentation. Review
authentication-header handling carefully, and place the service behind TLS and
appropriate access controls for production use.
