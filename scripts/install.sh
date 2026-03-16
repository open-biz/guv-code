#!/bin/sh
#
# GuvCode Installer
# "Right away, Guv'nor."
#
# Usage: curl -fsSL https://bit.ly/install-guv | sh

set -e

REPO="vovk/guv-code"
BINARY="guv"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
GUV_DIR="$HOME/.guv"

# --- Utility Functions ---

info() {
  printf "\033[34m[INFO]\033[0m %s\n" "$1"
}

success() {
  printf "\033[32m[SUCCESS]\033[0m %s\n" "$1"
}

error() {
  printf "\033[31m[ERROR]\033[0m %s\n" "$1" >&2
  exit 1
}

# --- System Detection ---

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS" in
  darwin) OS="darwin" ;;
  linux) OS="linux" ;;
  *) error "Unsupported operating system: $OS" ;;
esac

ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH="amd64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) error "Unsupported architecture: $ARCH" ;;
esac

# --- Version Fetching ---

info "Fetching latest version from GitHub..."
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
  # Fallback to a default if API fails or no releases yet
  VERSION="v0.1.0"
  info "Could not fetch latest version, falling back to $VERSION"
fi

VERSION_NUM="${VERSION#v}"

# --- Binary Download & Install ---

FILENAME="${BINARY}_${VERSION_NUM}_${OS}_${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

info "Installing ${BINARY} ${VERSION} (${OS}/${ARCH})..."

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

info "Downloading ${DOWNLOAD_URL}..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${FILENAME}"; then
  error "Failed to download binary. The release might not exist yet for your platform."
fi

info "Extracting..."
tar -xzf "${TMP_DIR}/${FILENAME}" -C "$TMP_DIR"

info "Installing to ${INSTALL_DIR}/${BINARY}..."
if [ -w "$INSTALL_DIR" ]; then
  mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
  info "Requesting sudo permissions to move binary to ${INSTALL_DIR}..."
  sudo mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

success "GuvCode CLI installed successfully!"

# --- Frontend Installation (Optional) ---

echo ""
info "GuvCode includes a Web Dashboard for account and usage management."
info "The cloud version is at: https://guv-code.vercel.app"
printf "Would you like to install the dashboard locally for development? (y/N): "
read -r INSTALL_WEB

if [ "$INSTALL_WEB" = "y" ] || [ "$INSTALL_WEB" = "Y" ]; then
    if ! command -v git >/dev/null 2>&1; then
        error "git is required to install the dashboard."
    fi

    info "Setting up GuvCode source in ${GUV_DIR}/source..."
    mkdir -p "$GUV_DIR"
    
    if [ -d "${GUV_DIR}/source" ]; then
        info "Source already exists, pulling latest changes..."
        cd "${GUV_DIR}/source" && git pull
    else
        git clone "https://github.com/${REPO}.git" "${GUV_DIR}/source"
    fi

    info "Checking for 'bun' (required for the web dashboard)..."
    if command -v bun >/dev/null 2>&1; then
        cd "${GUV_DIR}/source/web"
        info "Installing dependencies..."
        bun install
        success "Dashboard installed locally."
        info "You can start it by running: cd ${GUV_DIR}/source/web && bun run dev"
    else
        info "Bun not found. Please install Bun (https://bun.sh) and then run:"
        info "cd ${GUV_DIR}/source/web && bun install && bun run dev"
    fi
fi

echo ""
success "GuvCode is ready to go!"
echo "Run 'guv --help' to get started."
echo "Right away, Guv'nor."
