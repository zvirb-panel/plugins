# plugins

The first-party Zvirb game plugins: one Cargo workspace, one crate per game, each built to a Wasm module. Read the workspace `CLAUDE.md` and `CONSTITUTION.md` first (`../zvirb-workspace`); they outrank this file. The workspace has no members yet.

## Layout

| Path | Holds |
| --- | --- |
| `<plugin-id>/` | One crate per game, named by the plugin id (`minecraft-java`), added to `members` in the root `Cargo.toml`. Built like `../plugin-template`'s `example` crate: `crate-type = ["cdylib", "rlib"]`, `[lints] workspace = true` |
| `Cargo.toml` | The workspace, its shared package fields and `[workspace.lints]` (`clippy::pedantic` at warn) |
| `mise.toml` | Pinned Rust (with `wasm32-unknown-unknown`, `rustfmt`, `clippy`) and Node, plus the `check` and `fmt` tasks |
| `scripts/check.mjs` | Runs the checks, or prints `No plugins yet` while the workspace has no members |
| `scripts/header.mjs` | Prints the header that names each check step |

The plugin manifest and the PDK are not defined here. A crate takes them from the published PDK once it exists.

## Commands

| Command | Does |
| --- | --- |
| `mise install` | Installs the pinned tools |
| `mise run check` | With no members, prints `No plugins yet` and passes. Otherwise `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, then `cargo build --target wasm32-unknown-unknown --release` |
| `mise run fmt` | `cargo fmt` |

Modules land in `target/wasm32-unknown-unknown/release/`.

## Conventions

- Rust only. Match the surrounding file; `cargo fmt` is the formatter.
- No blanket lint allows. An individual `allow` goes in `Cargo.toml` or on the item with a comment saying why.
- Commits are Conventional Commits with the plugin id as scope, or `ci`, `docs`, `deps` (`commits` skill).
- Each plugin is released on its own line (`minecraft-java-v1.3.0`); a release opens a pull request to `plugin-registry`.
- Skills: `craft` always; `commits` for messages.
