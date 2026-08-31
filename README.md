# gpui-experimental

Experimental, modular application building blocks for [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).

The workspace is intentionally small and currently contains scaffolds for a global command
registry, preview providers, feature views, a default workspace shell, and a command palette.
The proposed boundaries and the parts that are implemented today are documented in
[ARCHITECTURE.md](./ARCHITECTURE.md).

## Crates

- `collections` provides shared collection aliases.
- `command` registers and executes application-level commands without requiring a window.
- `macros` provides shared derives, including the `Args` derive for command inputs.
- `preview` supplies fallback preview providers for searches with no direct command result.
- `view` owns logical view identity and view/window lifecycle.
- `workspace` hosts the default three-region page.
- `command-palette` renders the default command and preview search surface.

## Development

This repository has no binary target yet. Use the workspace checks while the crates are being
implemented:

```text
cargo fmt --all -- --check
cargo check --workspace
```
