# scaffolder

Multi-language project scaffolder. First template: TypeScript + Node.js.

## Install

### macOS / Linux

```sh
curl -fsSL https://raw.githubusercontent.com/reedchan7/scaffolder/main/install.sh | sh
```

This detects your OS and CPU architecture, downloads the matching prebuilt
binary, and installs it to `/usr/local/bin`. Re-run it any time to update.

Customize the version or install location:

```sh
curl -fsSL https://raw.githubusercontent.com/reedchan7/scaffolder/main/install.sh \
  | sh -s -- --version v0.1.0 --bin-dir "$HOME/.local/bin"
```

| Option | Env var | Default |
|--------|---------|---------|
| `--version <tag>` | `SCAFFOLDER_VERSION` | `latest` |
| `--bin-dir <dir>` | `SCAFFOLDER_INSTALL_DIR` | `/usr/local/bin` |

### Windows (PowerShell)

```powershell
irm https://github.com/reedchan7/scaffolder/releases/latest/download/scaffolder-installer.ps1 | iex
```

### Manual download

Grab a prebuilt archive from the [latest release][releases], extract it, and put
the `scaffolder` binary on your `PATH`:

| OS | x86_64 / AMD64 | aarch64 / ARM64 |
|----|----------------|-----------------|
| Linux | `scaffolder-x86_64-unknown-linux-gnu.tar.xz` | `scaffolder-aarch64-unknown-linux-gnu.tar.xz` |
| macOS | `scaffolder-x86_64-apple-darwin.tar.xz` | `scaffolder-aarch64-apple-darwin.tar.xz` |
| Windows | `scaffolder-x86_64-pc-windows-msvc.zip` | `scaffolder-aarch64-pc-windows-msvc.zip` |

[releases]: https://github.com/reedchan7/scaffolder/releases

## Update

Re-run the install command above, or use the built-in self-updater:

```sh
scaffolder self-update
```

## Usage

```sh
scaffolder new typescript-node my-app      # one-shot (CI friendly)
scaffolder new                             # interactive
scaffolder list
scaffolder self-update
```

### `new` options

| Flag | Default | Values |
|------|---------|--------|
| `--pm` | `pnpm` | `pnpm` `npm` `yarn` `bun` |
| `--test` | `vitest` | `vitest` `node` |
| `--module` | `esm` | `esm` `cjs` |
| `--node` | `24` | major version integer |
| `--license` | _(private)_ | `MIT` `Apache-2.0` |
| `--ai` | off | flag — also writes `CLAUDE.md` + `AGENTS.md` |
| `--no-git` | off | flag — skip `git init` |
| `--no-install` | off | flag — skip dependency install |

The generated project ships with TypeScript (ESNext), ESLint + Prettier, Lefthook
git hooks, a Makefile, and a test setup — ready to `make check`.

## Develop

Common tasks are wrapped in a `Makefile` — run `make help` to list them:

```sh
make build                              # release binary -> target/release/scaffolder
make test                               # unit + integration tests
make check                              # fmt + clippy + tests (what CI runs)
make install                            # build & install to /usr/local/bin (override: BINDIR=...)
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
