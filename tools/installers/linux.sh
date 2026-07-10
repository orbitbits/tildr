#!/usr/bin/env sh
# Tildr Installer
# Copyright (c) 2026 OrbitBits. All rights reserved.
# Author: William C. Canin <https://williamcanin.github.io>
#
# --- Usage:
# Help
# curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh -s -- --help
#
# List versions
# curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh -s -- --versions
#
# Install a specific version
# curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh -s -- 0.1.1
#
# Install the latest (no argument)
# curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh
#
# Uninstall
# curl -fsSL https://orbitbits.github.io/tildr/linux.sh | sh -s -- --uninstall
#
set -e
export PATH="/usr/local/bin:$PATH"

APP_NAME="Tildr"
BIN_NAME="tildr"
PKG_NAME="tildr"
REPO="orbitbits/tildr"
BRANCH="main"

API_LATEST="https://api.github.com/repos/${REPO}/releases/latest"
API_RELEASES="https://api.github.com/repos/${REPO}/releases?per_page=50"

ARCH="$(uname -m)"

INSTALLATION_DIR="/usr/local/bin"
MAN_DIR="/usr/share/man/man1"
LICENSE_DIR="/usr/share/licenses/${PKG_NAME}"
LICENSE_URL="https://raw.githubusercontent.com/${REPO}/refs/heads/${BRANCH}/LICENSE"

MAN_BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}/docs/man/dist"
MAN_PAGES="${PKG_NAME}.1
${PKG_NAME}-config.1
${PKG_NAME}-commands.1
${PKG_NAME}-security.1"

PLUGINS_BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}/tools/plugins"
NAUTILUS_VERSIONS="'4.1'" # using with multiple versions:: "'4.1', '4.0', '3.0'"
PLUGIN_NAUTILUS_DIR="/usr/share/nautilus-python/extensions"
PLUGIN_NAUTILUS_URL="${PLUGINS_BASE_URL}/nautilus/${PKG_NAME}.py"
PLUGIN_DOLPHIN_DIR="${HOME}/.local/share/kio/servicemenus"
PLUGIN_DOLPHIN_URL="${PLUGINS_BASE_URL}/dolphin/${PKG_NAME}.desktop"

# ----- UI -----
title() { printf "\033[0;35m[ %s ]\033[0m\n" "$1"; }
info() { printf "\033[0;36m-> %s\033[0m\n" "$1"; }
finish() { printf "\033[0;32m* %s\033[0m\n" "$1"; }
warning() { printf "\033[0;33m! %s\033[0m\n" "$1"; }
error() { printf "\033[0;31mx %s\033[0m\n" "$1"; }

# ----- Check Git installed -----
check_git() {
	if ! command -v git >/dev/null 2>&1; then
		warning "
Git was not found installed on your machine!
--------------------------------------------
This will not interfere with the use of ${APP_NAME}.

${APP_NAME} uses Git as a complement to provide a better
repository management experience."
	fi
}

# ----- Fetch (retry) -----
fetch_url() {
	url="$1"
	retries=3
	delay=1

	while [ "$retries" -gt 0 ]; do
		if command -v curl >/dev/null 2>&1; then
			curl -fsSL "$url" && return 0
		elif command -v wget >/dev/null 2>&1; then
			wget -qO- "$url" && return 0
		fi

		retries=$((retries - 1))
		sleep "$delay"
		delay=$((delay * 2))
	done

	return 1
}

# ----- Download (retry) -----
download_file() {
	url="$1"
	output="$2"

	retries=3
	delay=1

	while [ "$retries" -gt 0 ]; do
		if command -v curl >/dev/null 2>&1; then
			curl -L --fail --progress-bar "$url" -o "$output" && return 0
		else
			wget --show-progress "$url" -O "$output" && return 0
		fi

		retries=$((retries - 1))
		sleep "$delay"
		delay=$((delay * 2))
	done

	return 1
}

# ----- Check Python 3 -----
check_python3() {
	command -v python3 >/dev/null 2>&1
}

# ----- Check Nautilus Installed -----
check_nautilus_python() {

	if check_python3; then
		return 0
	fi

	info "Checking if Nautilus is installed...wait..."

	# Checks if typelib exists (works on all distros)
	if find /usr/lib /usr/lib64 /usr/local/lib -name "Nautilus-*.typelib" 2>/dev/null | grep -q .; then
		return 0
	fi

	# Checks if it imports via Python (final confirmation)
	if python3 -c "
import gi
for v in [${NAUTILUS_VERSIONS}]:
    try:
        gi.require_version('Nautilus', v)
        from gi.repository import Nautilus
        exit(0)
    except:
        pass
exit(1)
" 2>/dev/null; then
		return 0
	fi

	return 1
}

# ----- Plugin Nautilus -----
install_plugin_nautilus() {
	if check_nautilus_python; then
		info "Installing the ${APP_NAME} plugin for Nautilus (Nautilus-Python)..."

		TMP_PLUGIN=$(mktemp 2>/dev/null || echo "/tmp/${PKG_NAME}-plugin-nautilus.$$")

		if ! download_file "$PLUGIN_NAUTILUS_URL" "${TMP_PLUGIN}"; then
			warning "${APP_NAME} plugin for Nautilus file not found"
			return 0
		fi

		$SUDO mkdir -p "$PLUGIN_NAUTILUS_DIR"
		$SUDO mv "${TMP_PLUGIN}" "$PLUGIN_NAUTILUS_DIR/${PKG_NAME}.py"

		finish "${APP_NAME} plugin for Nautilus installed!"
	else
		warning "Nautilus-Python not installed!
--------------------------------
If you want to use the ${APP_NAME} plugin for Nautilus, install:

Arch Linux:          sudo pacman -S python-nautilus
Debian/Ubuntu/Mint:  sudo apt install python3-nautilus
Fedora:              sudo dnf install nautilus-python"
	fi
}

# ----- Check Dolphin Installed -----
check_dolphin() {
	command -v dolphin >/dev/null 2>&1
}

# ----- Plugin Dolphin -----
install_plugin_dolphin() {
	if check_dolphin; then
		info "Installing the ${APP_NAME} plugin for Dolphin (KDE ServiceMenu)..."

		TMP_PLUGIN=$(mktemp 2>/dev/null || echo "/tmp/${PKG_NAME}-plugin-dolphin.$$")

		if ! download_file "$PLUGIN_DOLPHIN_URL" "${TMP_PLUGIN}"; then
			warning "${APP_NAME} plugin for Dolphin file not found"
			return 0
		fi

		mkdir -p "${PLUGIN_DOLPHIN_DIR}"
		mv "${TMP_PLUGIN}" "${PLUGIN_DOLPHIN_DIR}/${PKG_NAME}.desktop"

		finish "${APP_NAME} plugin for Dolphin installed!"
	else
		warning "Dolphin not installed!
--------------------------------
If you want to use the ${APP_NAME} plugin for Dolphin, install:

Arch Linux:          sudo pacman -S dolphin
Debian/Ubuntu/Mint:  sudo apt install dolphin
Fedora:              sudo dnf install dolphin"
	fi
}

# ----- Checksum verify -----
verify_checksum() {
	file="$1"
	version="$2"

	if ! command -v sha256sum >/dev/null 2>&1; then
		warning "sha256sum not found, skipping verification"
		return 0
	fi

	SHA256SUMS_URL="https://github.com/${REPO}/releases/download/v${version}/SHA256SUMS"
	TMP_SHA=$(mktemp 2>/dev/null || echo "/tmp/SHA256SUMS.$$")

	if ! download_file "${SHA256SUMS_URL}" "${TMP_SHA}"; then
		warning "Checksum file not found, skipping verification"
		return 0
	fi

	EXPECTED=$(grep "$(basename "$file")" "${TMP_SHA}" | awk '{print $1}')
	ACTUAL=$(sha256sum "$file" | awk '{print $1}')

	rm -f "${TMP_SHA}"

	if [ -z "${EXPECTED}" ]; then
		warning "Checksum entry not found, skipping"
		return 0
	fi

	if [ "${EXPECTED}" != "${ACTUAL}" ]; then
		error "Checksum mismatch!"
		exit 1
	fi

	finish "Checksum verified"
}

# ----- License -----
install_license() {
	TMP_LICENSE=$(mktemp 2>/dev/null || echo "/tmp/license.$$")

	$SUDO mkdir -p "${LICENSE_DIR}"

	if ! download_file "$LICENSE_URL" "${TMP_LICENSE}"; then
		warning "LICENSE file not found"
		return 0
	fi

	$SUDO mv "${TMP_LICENSE}" "${LICENSE_DIR}/LICENSE"

	finish "LICENSE installed"
}

# ----- Man pages -----
install_man_pages() {
	info "Installing man pages"

	$SUDO mkdir -p "${MAN_DIR}"

	for page in $MAN_PAGES; do
		url="$MAN_BASE_URL/${page}"
		TMP_MAN=$(mktemp 2>/dev/null || echo "/tmp/page.$$")

		printf "  → %s\n" "${page}"

		if ! download_file "$url" "$TMP_MAN"; then
			warning "Failed to download ${page}"
			continue
		fi

		$SUDO mv "$TMP_MAN" "${MAN_DIR}/${page}"
	done

	if command -v mandb >/dev/null 2>&1; then
		$SUDO mandb -q 2>/dev/null || true
	elif command -v makewhatis >/dev/null 2>&1; then
		$SUDO makewhatis >/dev/null 2>&1
	fi

	finish "Man pages installed"
}

# ----- Privileges -----
if [ "$(id -u)" -eq 0 ]; then
	SUDO=""
else
	if ! command -v sudo >/dev/null 2>&1; then
		error "Requires root privileges (install sudo or run as root)"
		exit 1
	fi
	SUDO="sudo"
fi

# ----- OS / ARCH check -----
if [ "$(uname -s)" != "Linux" ]; then
	error "Linux only"
	exit 1
fi

case "$ARCH" in
x86_64 | amd64) ARCH="x86_64" ;;
aarch64 | arm64) ARCH="aarch64" ;;
*)
	error "Unsupported architecture: $ARCH"
	exit 1
	;;
esac

# -----Dependencies -----
if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
	error "curl or wget required"
	exit 1
fi

# ----- Args -----
ARG="${1:-}"

case "$ARG" in
--help | -h)
	echo "Usage:"
	echo "  install latest: curl ... | sh"
	echo "  install version: curl ... | sh -s -- 0.1.0"
	echo "  list versions: --versions"
	echo "  uninstall: --uninstall"
	exit 0
	;;

--versions)
	title "Available versions"
	fetch_url "${API_RELEASES}" | sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p'
	exit 0
	;;

--uninstall)
	title "Uninstall"

	if [ -f "${INSTALLATION_DIR}/${BIN_NAME}" ]; then
		$SUDO rm -f "${INSTALLATION_DIR}/${BIN_NAME}"
		finish "Binary removed"
	fi

	$SUDO rm -f "${PLUGIN_NAUTILUS_DIR}/${PKG_NAME}.py"
	rm -f "${PLUGIN_DOLPHIN_DIR}/${PKG_NAME}.desktop"
	finish "Plugins removed"

	for page in $MAN_PAGES; do
		$SUDO rm -f "${MAN_DIR}/${page}"
	done

	finish "Man pages removed"

	$SUDO rm -f "${LICENSE_DIR}/LICENSE"

	finish "LICENSE removed"
	finish "Done!"

	exit 0
	;;

"")
	data="$(fetch_url "${API_LATEST}")" || exit 1
	VERSION_TAG=$(printf "%s" "${data}" | sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p')
	;;

*)
	VERSION_TAG="${ARG#v}"
	;;
esac

# ----- Download URLs -----
PRIMARY_URL="https://github.com/${REPO}/releases/download/v${VERSION_TAG}/${BIN_NAME}-${VERSION_TAG}-linux-${ARCH}"
# FALLBACK_URL="https://github.com/${REPO}/releases/download/v${VERSION_TAG}/${BIN_NAME}-${VERSION_TAG}-linux-${ARCH}"

title "Installing ${APP_NAME} ${VERSION_TAG}"

TMPFILE=$(mktemp 2>/dev/null || echo "/tmp/${PKG_NAME}.$$")
trap 'rm -f "${TMPFILE}"' EXIT

# download with fallback
if ! download_file "${PRIMARY_URL}" "${TMPFILE}"; then
	# warning "Primary failed, trying fallback"
	# if ! download_file "$FALLBACK_URL" "${TMPFILE}"; then
	error "Download failed"
	exit 1
	# fi
fi

finish "Download complete"

# checksum
verify_checksum "${TMPFILE}" "${VERSION_TAG}"

# ----- Install (atomic) -----
TMP_INSTALL=$(mktemp 2>/dev/null || echo "/tmp/${PKG_NAME}-install.$$")

cp "${TMPFILE}" "$TMP_INSTALL"
chmod +x "$TMP_INSTALL"

$SUDO mkdir -p "${INSTALLATION_DIR}"
$SUDO mv "$TMP_INSTALL" "${INSTALLATION_DIR}/${BIN_NAME}"

finish "Binary installed"

# ----- Validate -----
if ! command -v "${BIN_NAME}" >/dev/null 2>&1; then
	error "Binary not found in PATH"
	exit 1
fi

finish "Installation OK"

# ----- Plugins -----
install_plugin_nautilus
install_plugin_dolphin

# ----- Man pages -----
install_man_pages

# ----- LICENSE -----
install_license

# ----- Git -----
check_git

finish "Done!"
