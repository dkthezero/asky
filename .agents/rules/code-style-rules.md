---
trigger: always_on
---

# Rust + Tokio Agent Rules

## Project Context
You are an expert Rust developer working with Tokio runtime. This project is a CLI.

## Code Style & Structure
### Rust Defaults
- Embrace ownership and borrowing. Prefer borrowing (`&T`, `&mut T`) over cloning unless necessary.
- Use `Result<T, E>` for fallible operations and `Option<T>` for optional values — propagate errors with `?` operator instead of `unwrap()`.
- Use `impl Trait` in function signatures for return types and `&dyn Trait` for dynamic dispatch — prefer generics over trait objects when possible.
- Use `?` operator for error propagation. Define custom error types with `thiserror` or implement `std::error::Error`.
- Prefer iterators and combinators (`.map()`, `.filter()`, `.collect()`) over manual loops.
- Use `clippy` lints and fix all warnings. Run `cargo fmt` before every commit.
- Use `#[derive(...)]` for common traits: `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.
- Prefer `&str` over `String` in function parameters; return `String` when ownership transfer is needed.

### Tokio Async Runtime
- Use `#[tokio::main]` for the async entry point and `tokio::spawn` for concurrent tasks.
- Use `tokio::fs`, `tokio::net`, `tokio::io` for async I/O instead of `std::fs`, `std::net`.
- Use `tokio::select!` for racing multiple async operations with cancellation.
- Use `tokio::sync` primitives (`Mutex`, `RwLock`, `mpsc`, `oneshot`) for async-safe synchronization.
- Avoid blocking operations in async context — use `tokio::task::spawn_blocking` for CPU-heavy work.
- Use `tokio::time::timeout` to wrap operations that might hang indefinitely.
- Use `tokio::signal` for graceful shutdown handling in servers.
- Prefer `tokio::sync::mpsc` channels over shared state for inter-task communication.
- Use `tokio::task::JoinSet` for managing groups of spawned tasks.

### Rust API Guidelines
- Use snake_case for functions, variables, and modules with descriptive names including auxiliary verbs (e.g., is_valid, has_error).
- Handle errors early using guard clauses, early returns, and the ? operator.
- Minimize allocations in hot paths; prefer zero-copy operations and static data where possible.
- Modularize code to avoid duplication, favoring iteration over repetition.
- Separate policy and metadata management from core storage for cleaner APIs.
- Prefer contiguous storage with index-based indirection over scattered pointers or dynamic structures.
- Design concurrency explicitly from the start (e.g., sharding or lock-free) rather than as an afterthought.
- Document all public items with `///` doc comments — include a `# Examples` section with a runnable `doctest` and `# Errors` / `# Panics` / `# Safety` sections where applicable.
- Implement structured logging with contextual fields for better observability.

### Mixed Patterns
- Use functional patterns for data transformations and pure logic.
- Choose the paradigm that best fits each module's responsibility.

## Linting & Formatting
### Rustfmt & Clippy
- Run `cargo fmt` before every commit. Configure in `rustfmt.toml` if needed.
- Run `cargo clippy` and fix all warnings — Clippy catches common mistakes and unidiomatic code.
- Run `cargo clippy -- -W clippy::all` for comprehensive linting and `cargo fmt` for formatting — add both to CI.
- Use `cargo clippy -- -D warnings` in CI to treat warnings as errors.
- Use `#[allow(clippy::lint_name)]` for intentional suppressions — always add a comment explaining why.
- Configure `rustfmt.toml` for team preferences: `max_width`, `use_field_init_shorthand`, `edition`.
- Run `cargo clippy --all-targets --all-features` to lint test code and feature-gated code too.

## Architecture
### CLI Architecture
- Structure CLI apps with a clear command → handler → output pipeline. Separate argument parsing from business logic.
- Use subcommands for complex CLIs. Each subcommand should have its own help text, flags, and validation.
- Exit with meaningful codes: 0 for success, 1 for general errors, 2 for usage errors. Document exit codes.
- Write to stdout for output data, stderr for logs/progress/errors. This enables piping and redirection.
- Implement a config hierarchy: CLI flags > env vars > config file > defaults. Use XDG directories for config files.
- Add --json or --output=json flag for machine-readable output. Human-readable by default, structured when piped.
- Validate all inputs early and fail fast with clear error messages that include the invalid value and expected format.
- Support --verbose/-v and --quiet/-q flags. Default output should be minimal but informative.
- Add shell completion scripts (bash, zsh, fish). Most CLI frameworks generate these automatically.
- Use progress bars for long operations. Detect TTY and suppress progress in non-interactive mode.

## Performance
### Rust Performance
- Use `&str` and `&[T]` (borrowed slices) to avoid unnecessary cloning and allocation.
- Compile with `--release` for optimized builds. Debug builds are 10-100x slower.
- Use `cargo bench` with Criterion.rs for benchmarks — compare against baselines to detect regressions across commits.
- Use iterators and combinators instead of indexed loops — they often optimize to the same assembly.
- Use `Vec::with_capacity(n)` when the final size is known to avoid reallocation.
- Use `Cow<str>` for functions that sometimes need to allocate and sometimes can borrow.
- Use `rayon` for data parallelism: `.par_iter()` for parallel map/filter/reduce.

## Testing
### Rust Testing
- Use `#[cfg(test)]` module in each source file for unit tests. Use `assert_eq!`, `assert_ne!`, `assert!` macros. Put integration tests in `tests/` directory — each file is a separate test binary.
- Use `#[should_panic(expected = "message")]` for testing error conditions. Use `Result<(), Box<dyn Error>>` as test return type for `?` operator in tests. Use `cargo test -- --nocapture` to see stdout. Organize test helpers in `tests/common/mod.rs`. Use `#[ignore]` for slow tests, run with `cargo test -- --ignored`.

### Unit Testing
- Write unit tests for every new function or method immediately after implementation.
- Run the full unit test suite before committing — never push code with failing tests.
- Test one behavior per test case. Keep tests fast, isolated, and deterministic.
- Follow the Arrange-Act-Assert pattern: set up inputs, call the function, verify the output.
- Mock external dependencies (APIs, databases, file system) — unit tests validate your logic in isolation.
- Name tests descriptively: `should return empty array when no items match filter`.
- Test edge cases: empty inputs, nulls, boundary values, error conditions — not just the happy path.
- Run unit tests after every code change during development for fast feedback.

## Libraries & Tools
### Tokio
- Use `#[tokio::main]` on `async fn main()` and `#[tokio::test]` on async test functions — Tokio's runtime handles the async executor.
- Use `tokio::spawn()` for concurrent tasks and `tokio::select!` for racing multiple futures — both are zero-cost abstractions over the runtime.
- Use `tokio::sync::Mutex` (not `std::sync::Mutex`) for shared state in async code — std Mutex blocks the thread, Tokio Mutex yields to the runtime.
- Use `tokio::sync::Mutex` (not `std::sync::Mutex`) for async-safe locking.
- Use `tokio::sync::mpsc` for multi-producer channels, `oneshot` for single-response patterns.
- Use `tokio::time::timeout()` to bound async operations. Never let operations hang indefinitely.
- Use `tokio::fs` for async file operations instead of blocking `std::fs` on the async runtime.

### Clap
- Use derive macros: `#[derive(Parser)]` on a struct with `#[arg()]` attributes on fields — Clap generates the CLI parser at compile time.
- Use `#[command(about, version)]` for auto-generated help text and `#[arg(short, long, default_value)]` for flag configuration.
- Use subcommands with `#[derive(Subcommand)]` enum and `#[command(subcommand)]` field on the main struct — pattern match in `main()`.
- Use `#[arg(default_value_t = ...)]` for defaults, `#[arg(value_enum)]` for enum arguments.
- Use `#[command(about, version, author)]` for auto-generated help text.
- Group related arguments into subcommand structs with `#[derive(Subcommand)]`.
- Use `#[arg(env = "MY_VAR")]` to allow arguments from environment variables.


