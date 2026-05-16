#!/usr/bin/env bash
# geek-cli installer
#   curl -fsSL https://yangtzeu.work/geek/install.sh | sh
#   curl -fsSL https://github.com/Yangtze-University-Geek-Class/geek-cli/releases/latest/download/install.sh | sh
#
# Env overrides:
#   GEEK_VERSION=v0.1.2     # pin version (default: latest)
#   GEEK_INSTALL_DIR=$HOME/.local/bin
#   GEEK_BINARY_URL=...     # bypass detection
set -euo pipefail

REPO="Yangtze-University-Geek-Class/geek-cli"
VERSION="${GEEK_VERSION:-latest}"
INSTALL_DIR="${GEEK_INSTALL_DIR:-$HOME/.local/bin}"

err()  { printf '\033[31m[geek-cli]\033[0m %s\n' "$*" >&2; }
info() { printf '\033[36m[geek-cli]\033[0m %s\n' "$*"; }
ok()   { printf '\033[32m[geek-cli]\033[0m %s\n' "$*"; }

# -- platform detect ----------------------------------------------------------
os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$os-$arch" in
  darwin-arm64|darwin-aarch64) target="aarch64-apple-darwin" ;;
  darwin-x86_64|darwin-amd64)  target="x86_64-apple-darwin" ;;
  linux-aarch64|linux-arm64)   target="aarch64-unknown-linux-gnu" ;;
  linux-x86_64|linux-amd64)
    if ldd --version 2>&1 | grep -qi musl; then
      target="x86_64-unknown-linux-musl"
    else
      target="x86_64-unknown-linux-gnu"
    fi
    ;;
  *) err "unsupported platform: $os-$arch"; exit 1 ;;
esac
binary_name="geek-${target}"

# -- resolve version ----------------------------------------------------------
if [ "$VERSION" = "latest" ]; then
  base="https://github.com/${REPO}/releases/latest/download"
else
  base="https://github.com/${REPO}/releases/download/${VERSION}"
fi
url="${GEEK_BINARY_URL:-$base/$binary_name}"
sha_url="$base/$binary_name.sha256"

# -- download -----------------------------------------------------------------
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
info "downloading $binary_name from $url"
if ! curl -fL --retry 3 --retry-delay 2 -o "$tmp/geek" "$url"; then
  err "download failed; check network or release availability"
  exit 1
fi

# -- verify sha256 (best-effort, doesn't fail install) -----------------------
if curl -fsSL "$sha_url" -o "$tmp/geek.sha256" 2>/dev/null; then
  want="$(awk '{print $1}' "$tmp/geek.sha256")"
  if command -v sha256sum >/dev/null 2>&1; then
    got="$(sha256sum "$tmp/geek" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    got="$(shasum -a 256 "$tmp/geek" | awk '{print $1}')"
  else
    got=""
  fi
  if [ -n "$got" ] && [ "$got" != "$want" ]; then
    err "sha256 mismatch (want $want, got $got)"
    exit 1
  fi
  [ -n "$got" ] && ok "sha256 verified"
fi

# -- install ------------------------------------------------------------------
mkdir -p "$INSTALL_DIR"
mv "$tmp/geek" "$INSTALL_DIR/geek"
chmod 0755 "$INSTALL_DIR/geek"
ok "installed: $INSTALL_DIR/geek"

# -- PATH check ---------------------------------------------------------------
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    info "add this to your shell rc to use \`geek\`:"
    info "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

# -- sanity -------------------------------------------------------------------
if "$INSTALL_DIR/geek" --version >/dev/null 2>&1; then
  ok "$("$INSTALL_DIR/geek" --version)"
fi
ok "next: geek login"
