# scaffolder

> A fast, opinionated project scaffolder that bootstraps production-ready codebases in seconds.

[![Release](https://github.com/reedchan7/scaffolder/actions/workflows/release.yml/badge.svg)](https://github.com/reedchan7/scaffolder/actions/workflows/release.yml)
[![Latest release](https://img.shields.io/github/v/release/reedchan7/scaffolder?sort=semver&logo=github)](https://github.com/reedchan7/scaffolder/releases/latest)
[![License](https://img.shields.io/github/license/reedchan7/scaffolder)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue)](#install)

`scaffolder` generates clean, batteries-included project templates from a single command —
correctly wired tooling, sensible defaults, and zero boilerplate to copy-paste. It ships as a
single static binary with no runtime dependencies.

The first template targets **TypeScript** with either a Node.js or Bun runtime; more languages are
on the way.

## Highlights

- **One command, ready to ship** — TypeScript (ESNext), ESLint + Prettier, Lefthook git hooks,
  a `Makefile`, and tests, all pre-wired.
- **Interactive or scripted** — guided prompts for humans, flags for CI.
- **Pick your stack** — choose your package manager, runtime-aware test runner, module system, and
  Node version when using the Node.js runtime.
- **Cross-platform** — prebuilt binaries for macOS, Linux, and Windows (x86_64 & ARM64).
- **Self-updating** — `scaffolder self-update` keeps you current.

## Install

### macOS / Linux

```sh
curl -fsSL https://raw.githubusercontent.com/reedchan7/scaffolder/main/install.sh | sh
```

This detects your OS and CPU architecture, downloads the matching prebuilt binary, and installs
it to `~/.local/bin`. Re-run it any time to update.

Customize the version or install location:

```sh
curl -fsSL https://raw.githubusercontent.com/reedchan7/scaffolder/main/install.sh \
  | sh -s -- --version v0.1.0 --bin-dir "$HOME/.local/bin"
```

| Option | Env var | Default |
|--------|---------|---------|
| `--version <tag>` | `SCAFFOLDER_VERSION` | `latest` |
| `--bin-dir <dir>` | `SCAFFOLDER_INSTALL_DIR` | `~/.local/bin` |

### Windows (PowerShell)

```powershell
irm https://github.com/reedchan7/scaffolder/releases/latest/download/scaffolder-installer.ps1 | iex
```

### Manual download

Grab a prebuilt archive from the [latest release][releases], extract it, and put the `scaffolder`
binary on your `PATH`:

| OS | x86_64 / AMD64 | aarch64 / ARM64 |
|----|----------------|-----------------|
| Linux | `scaffolder-x86_64-unknown-linux-gnu.tar.xz` | `scaffolder-aarch64-unknown-linux-gnu.tar.xz` |
| macOS | `scaffolder-x86_64-apple-darwin.tar.xz` | `scaffolder-aarch64-apple-darwin.tar.xz` |
| Windows | `scaffolder-x86_64-pc-windows-msvc.zip` | `scaffolder-aarch64-pc-windows-msvc.zip` |

[releases]: https://github.com/reedchan7/scaffolder/releases

## Quick start

```sh
scaffolder new typescript-node my-app      # one-shot (CI friendly)
scaffolder new                             # interactive
scaffolder list                            # show available templates
```

The generated project is ready to `make check` immediately — formatting, linting, and tests all
pass out of the box.

## Usage

```sh
scaffolder new [TEMPLATE] [NAME] [OPTIONS]   # scaffold a new project
scaffolder list                              # list available templates
scaffolder self-update                       # update to the latest release
```

### `new` options

| Flag | Default | Values |
|------|---------|--------|
| `--pm` | `pnpm` | `pnpm` `npm` `yarn` `bun` |
| `--test` | `vitest` (`bun` when `--pm bun`) | `vitest` `node` `bun` |
| `--module` | `esm` | `esm` `cjs` |
| `--node` | `24` | major version integer; used by Node.js projects |
| `--dir` | `.` | parent directory for the generated project |
| `--license` | _(private)_ | `MIT` `Apache-2.0` |
| `--ai` | off | flag — also writes `CLAUDE.md` + `AGENTS.md` |
| `--no-git` | off | flag — skip `git init` |
| `--no-install` | off | flag — skip dependency install |

> Omit `--license` to keep the project private (no license file is written).
>
> `--pm bun` generates a Bun-runtime project: Bun scripts, Bun TypeScript settings, and Bun's
> built-in test runner. `--test bun` is only valid with `--pm bun`.

## Update

Re-run the install command above, or use the built-in self-updater:

```sh
scaffolder self-update
```

## Contributing

Common tasks are wrapped in a `Makefile` — run `make help` to list them all:

```sh
make build                              # release binary -> target/release/scaffolder
make test                               # unit + integration tests
make check                              # fmt + clippy + tests (what CI runs)
make install                            # build & install to ~/.local/bin (override: BINDIR=...)
make run ARGS="new typescript-node demo"
```

### Cutting a release

```sh
make bump VERSION=0.2.0                  # update the version in Cargo.toml
git commit -am "chore: release v0.2.0"
make release                            # tag v0.2.0 and push -> CI builds & publishes
```

## License

[MIT](LICENSE) © Reed Chan
