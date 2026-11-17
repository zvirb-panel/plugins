# plugins

The first-party Zvirb game plugins: one Cargo workspace, one crate per game, each built to a Wasm module. Read the workspace `CLAUDE.md` and `.claude/rules/` first (`../zvirb-workspace`); they outrank this file.

## Layout

| Path | Holds |
| --- | --- |
| `<plugin-id>/` | One crate per game, named by the plugin id (`minecraft-java`), added to `members` in the root `Cargo.toml`. Built like `../plugin-template`'s `example` crate: `crate-type = ["cdylib", "rlib"]`, `[lints] workspace = true` |
| `sample/` | A one-engine crate that proves a plugin builds against the published PDK. Its test validates `zvirb-plugin.json` with the `jsonschema` crate against the output of `zvirb-pdk schema` |
| `Cargo.toml` | The workspace, its shared package fields and `[workspace.lints]` (`clippy::pedantic` at warn) |
| `mise.toml` | Pinned Rust (with `wasm32-unknown-unknown`, `rustfmt`, `clippy`) and Node, plus the `check` and `fmt` tasks |
| `scripts/check.mjs` | Runs the checks, or prints `No plugins yet` while the workspace has no members. Installs `zvirb-pdk` into `target/tools` at the commit `Cargo.lock` locks |
| `.cargo/config.toml` | Fetches the private PDK repository with the `git` command, using the credentials `gh auth setup-git` installs |
| `scripts/check-actions.mjs` | Fails on a workflow `uses:` that isn't pinned to a commit SHA |
| `scripts/check-advisories.mjs`, `osv-scanner.toml` | Runs `osv-scanner` over the lockfiles. An accepted advisory is an `[[IgnoredVulns]]` entry with `id`, `ignoreUntil` and `reason`. A lockfile entry without a `resolved` URL is not scanned. |
| `.github/workflows/release.yml` | release-please keeps a release pull request per plugin; merging one tags `<plugin>-v<version>`, and the `publish` job builds and packs that plugin, attaches the archive to the release, and opens a pull request to `plugin-registry` adding the version to the signed index. It reads `RELEASE_APP_CLIENT_ID` and the secrets `RELEASE_APP_PRIVATE_KEY` (the `zvirb-release` app, which reaches the PDK and the registry) and `REGISTRY_SIGNING_KEY`. Checks still run locally only, through `mise run check` |
| `release-please-config.json`, `.release-please-manifest.json` | One package per plugin, with `zvirb-plugin.json`'s version bumped beside `Cargo.toml`'s |
| `scripts/pack.mjs <plugin> <archive>` | Packs a built plugin as the archive Zvirb installs, a tar.gz with its manifest, the files that names, `points/` and `plugin.wasm` at the root, and prints its index entry for `plugin-registry`'s `add-version.mjs`. The summary is the crate's `description` |
| `scripts/header.mjs` | Prints the header that names each check step |

The plugin manifest and the PDK are not defined here. A crate takes them from the published PDK as a git dependency on a `pdk-v<version>` tag, never a path into `../zvirb`. `Cargo.lock` is committed and every cargo step runs with `--locked`.

## Commands

| Command | Does |
| --- | --- |
| `mise install` | Installs the pinned tools |
| `mise run check` | Pinned actions and `osv-scanner` (with no lockfile it reports `No package sources found` and passes), then, with no members, prints `No plugins yet` and passes. Otherwise `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo install` of `zvirb-pdk`, `cargo test --locked`, then `cargo build --locked --target wasm32-unknown-unknown --release` |
| `mise run fmt` | `cargo fmt` |

Modules land in `target/wasm32-unknown-unknown/release/`.

## Conventions

- Rust only. Match the surrounding file; `cargo fmt` is the formatter.
- No blanket lint allows. An individual `allow` goes in `Cargo.toml` or on the item with a comment saying why.
- Commits are Conventional Commits with the plugin id as scope, or `ci`, `docs`, `deps` (`commits` skill).
- Each plugin is released on its own line (`minecraft-java-v1.3.0`); a release opens a pull request to `plugin-registry`.
- Skills: `craft` always; `commits` for messages.
