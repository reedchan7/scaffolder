#!/bin/sh
# scaffolder installer
#
# Detects your OS and CPU architecture, downloads the matching prebuilt
# binary from GitHub Releases, and installs it (default: /usr/local/bin).
# Re-running this script updates an existing install in place.
#
# Quick start:
#   curl -fsSL https://raw.githubusercontent.com/reedchan7/scaffolder/main/install.sh | sh
#
# Options (pass after `sh -s --` when piping):
#   --version <tag>    install a specific release (e.g. v0.1.0); default: latest
#   --bin-dir  <dir>   install location;            default: /usr/local/bin
#   --help             show this help
#
# Environment overrides:
#   SCAFFOLDER_VERSION       same as --version
#   SCAFFOLDER_INSTALL_DIR   same as --bin-dir
#
# Windows: use the PowerShell installer instead (see README).
set -eu

REPO="reedchan7/scaffolder"
BIN="scaffolder"
INSTALL_DIR="${SCAFFOLDER_INSTALL_DIR:-/usr/local/bin}"
VERSION="${SCAFFOLDER_VERSION:-latest}"

info() { printf '  %s\n' "$*" >&2; }
warn() { printf 'warning: %s\n' "$*" >&2; }
err()  { printf 'error: %s\n' "$*" >&2; exit 1; }

usage() {
  sed -n '2,20p' "$0" 2>/dev/null | sed 's/^# \{0,1\}//'
  exit 0
}

parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --version) VERSION="${2:?--version needs a value}"; shift 2 ;;
      --bin-dir) INSTALL_DIR="${2:?--bin-dir needs a value}"; shift 2 ;;
      --help|-h) usage ;;
      *) err "unknown option: $1 (try --help)" ;;
    esac
  done
}

detect_target() {
  os="$(uname -s)"
  case "$os" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *) err "unsupported OS '$os'. Windows users: use the PowerShell installer (see README)." ;;
  esac

  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64)   arch="x86_64" ;;
    arm64|aarch64)  arch="aarch64" ;;
    *) err "unsupported architecture '$arch'." ;;
  esac

  echo "${arch}-${os}"
}

download() {
  _url="$1"; _out="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$_url" -o "$_out"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$_out" "$_url"
  else
    err "need 'curl' or 'wget' to download files."
  fi
}

verify_checksum() {
  _file="$1"; _sumfile="$2"
  [ -s "$_sumfile" ] || { warn "no checksum published; skipping verification."; return 0; }
  _expected="$(awk '{print $1; exit}' "$_sumfile")"
  if command -v sha256sum >/dev/null 2>&1; then
    _actual="$(sha256sum "$_file" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    _actual="$(shasum -a 256 "$_file" | awk '{print $1}')"
  else
    warn "no sha256 tool found; skipping verification."; return 0
  fi
  [ "$_expected" = "$_actual" ] || err "checksum mismatch (expected $_expected, got $_actual)."
  info "checksum verified."
}

as_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    sudo "$@"
  else
    err "cannot write to $INSTALL_DIR and 'sudo' is unavailable; re-run with --bin-dir <writable dir>."
  fi
}

install_binary() {
  _src="$1"
  _dest="$INSTALL_DIR/$BIN"
  chmod +x "$_src"
  if [ -d "$INSTALL_DIR" ] && [ -w "$INSTALL_DIR" ]; then
    mv -f "$_src" "$_dest"
  elif [ ! -e "$INSTALL_DIR" ] && mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    mv -f "$_src" "$_dest"
  else
    info "elevated permission needed to write $INSTALL_DIR"
    as_root mkdir -p "$INSTALL_DIR"
    as_root mv -f "$_src" "$_dest"
  fi
  echo "$_dest"
}

main() {
  parse_args "$@"

  target="$(detect_target)"
  archive="${BIN}-${target}.tar.xz"
  if [ "$VERSION" = "latest" ]; then
    base="https://github.com/${REPO}/releases/latest/download"
  else
    base="https://github.com/${REPO}/releases/download/${VERSION}"
  fi

  info "platform: ${target}"
  info "release:  ${VERSION}"
  info "target:   ${INSTALL_DIR}/${BIN}"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT INT TERM

  info "downloading ${archive} ..."
  download "${base}/${archive}" "${tmp}/${archive}" \
    || err "download failed. Does a release exist for ${VERSION} / ${target}? See https://github.com/${REPO}/releases"
  download "${base}/${archive}.sha256" "${tmp}/${archive}.sha256" 2>/dev/null || true
  verify_checksum "${tmp}/${archive}" "${tmp}/${archive}.sha256"

  tar -xf "${tmp}/${archive}" -C "$tmp"
  binpath="$(find "$tmp" -type f -name "$BIN" 2>/dev/null | head -n1)"
  [ -n "$binpath" ] || err "could not find '${BIN}' inside the downloaded archive."

  dest="$(install_binary "$binpath")"

  info ""
  info "installed: $("$dest" --version 2>/dev/null || echo "$BIN") -> $dest"
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) : ;;
    *) warn "${INSTALL_DIR} is not on your PATH. Add it, e.g.:"
       warn "  export PATH=\"${INSTALL_DIR}:\$PATH\"" ;;
  esac
  info "run 'scaffolder --help' to get started."
}

main "$@"
