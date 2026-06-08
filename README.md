# scaffolder

Multi-language project scaffolder. First template: TypeScript + Node.js.

## Install

macOS / Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/reedchan7/ts-scaffold/releases/latest/download/scaffolder-installer.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/reedchan7/ts-scaffold/releases/latest/download/scaffolder-installer.ps1 | iex
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

```sh
cargo test          # unit + integration
cargo run -- new typescript-node demo
```
