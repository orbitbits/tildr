#!/usr/bin/env sh
# Tildr Installer — macOS
# Copyright (c) 2026 OrbitBits. All rights reserved.
# Author: William C. Canin <https://williamcanin.github.io>
#
# --- Usage:
# Help
# curl -fsSL https://orbitbits.github.io/tildr/macos.sh | sh -s -- --help
#
# List versions
# curl -fsSL https://orbitbits.github.io/tildr/macos.sh | sh -s -- --versions
#
# Install a specific version
# curl -fsSL https://orbitbits.github.io/tildr/macos.sh | sh -s -- 0.1.1
#
# Install the latest (no argument)
# curl -fsSL https://orbitbits.github.io/tildr/macos.sh | sh
#
# Uninstall
# curl -fsSL https://orbitbits.github.io/tildr/macos.sh | sh -s -- --uninstall
#
set -e
export PATH="/usr/local/bin:/opt/homebrew/bin:$PATH"

APP_NAME="Tildr"
BIN_NAME="tildr"
PKG_NAME="tildr"
REPO="orbitbits/tildr"
BRANCH="main"

API_LATEST="https://api.github.com/repos/${REPO}/releases/latest"
API_RELEASES="https://api.github.com/repos/${REPO}/releases?per_page=50"

ARCH="$(uname -m)"

if [ -d "/opt/homebrew" ]; then
  BREW_PREFIX="/opt/homebrew"
elif [ -d "/usr/local/Homebrew" ] || [ -d "/usr/local/bin" ]; then
  BREW_PREFIX="/usr/local"
else
  BREW_PREFIX="/usr/local"
fi

INSTALLATION_DIR="${BREW_PREFIX}/bin"
MAN_DIR="${BREW_PREFIX}/share/man/man1"
LICENSE_DIR="${BREW_PREFIX}/share/doc/${PKG_NAME}"
LICENSE_URL="https://raw.githubusercontent.com/${REPO}/refs/heads/${BRANCH}/LICENSE"

MAN_BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}/docs/man/dist"
MAN_PAGES="${PKG_NAME}.1
${PKG_NAME}-config.1
${PKG_NAME}-commands.1
${PKG_NAME}-security.1"

# ----- UI -----
title()   { printf "\033[0;35m[ %s ]\033[0m\n" "$1"; }
info()    { printf "\033[0;36m-> %s\033[0m\n" "$1"; }
finish()  { printf "\033[0;32m* %s\033[0m\n" "$1"; }
warning() { printf "\033[0;33m! %s\033[0m\n" "$1"; }
error()   { printf "\033[0;31mx %s\033[0m\n" "$1"; }

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
		fi

		retries=$((retries - 1))
		sleep "$delay"
		delay=$((delay * 2))
	done

	return 1
}

# ----- Checksum verify -----
verify_checksum() {
	file="$1"
	version="$2"

	# macOS uses shasum instead of sha256sum
	if ! command -v shasum >/dev/null 2>&1; then
		warning "shasum not found, skipping verification"
		return 0
	fi

	SHA256SUMS_URL="https://github.com/${REPO}/releases/download/v${version}/SHA256SUMS"
	TMP_SHA=$(mktemp 2>/dev/null || echo "/tmp/SHA256SUMS.$$")

	if ! download_file "${SHA256SUMS_URL}" "${TMP_SHA}"; then
		warning "Checksum file not found, skipping verification"
		return 0
	fi

	EXPECTED=$(grep "$(basename "$file")" "${TMP_SHA}" | awk '{print $1}')
	ACTUAL=$(shasum -a 256 "$file" | awk '{print $1}')

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

	$SUDO /usr/libexec/makewhatis "${BREW_PREFIX}/share/man" >/dev/null 2>&1 || true
	finish "Man pages installed"
}

# ----- Privileges -----
if [ "$(id -u)" -eq 0 ]; then
	SUDO=""
else
	if ! command -v sudo >/dev/null 2>&1; then
		error "Requires root privileges (run as root)"
		exit 1
	fi
	SUDO="sudo"
fi

# ----- OS check -----
if [ "$(uname -s)" != "Darwin" ]; then
	error "macOS only"
	exit 1
fi

# ----- Arch detection -----
case "$ARCH" in
x86_64)           ARCH="x86_64" ;;
arm64 | aarch64)  ARCH="aarch64" ;;
*)
	error "Unsupported architecture: $ARCH"
	exit 1
	;;
esac

# ----- Dependencies -----
if ! command -v curl >/dev/null 2>&1; then
	error "curl is required"
	exit 1
fi

# ----- Args -----
ARG="${1:-}"

case "$ARG" in
--help | -h)
	echo "Usage:"
	echo "  install latest:   curl ... | sh"
	echo "  install version:  curl ... | sh -s -- 0.1.1"
	echo "  list versions:    curl ... | sh -s -- --versions"
	echo "  uninstall:        curl ... | sh -s -- --uninstall"
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

# ----- Download URL -----
PRIMARY_URL="https://github.com/${REPO}/releases/download/v${VERSION_TAG}/${BIN_NAME}-${VERSION_TAG}-macos-${ARCH}"

title "Installing ${APP_NAME} ${VERSION_TAG} (macOS ${ARCH})"

TMPFILE=$(mktemp 2>/dev/null || echo "/tmp/${PKG_NAME}.$$")
trap 'rm -f "${TMPFILE}"' EXIT

if ! download_file "${PRIMARY_URL}" "${TMPFILE}"; then
	error "Download failed"
	exit 1
fi

finish "Download complete"

# ----- Checksum -----
verify_checksum "${TMPFILE}" "${VERSION_TAG}"

# ----- Install (atomic) -----
TMP_INSTALL=$(mktemp 2>/dev/null || echo "/tmp/${PKG_NAME}-install.$$")

cp "${TMPFILE}" "$TMP_INSTALL"
chmod +x "$TMP_INSTALL"

$SUDO mkdir -p "${INSTALLATION_DIR}"
$SUDO mv "$TMP_INSTALL" "${INSTALLATION_DIR}/${BIN_NAME}"

finish "Binary installed"

# ----- Remove macOS quarantine -----
# macOS blocks downloaded binaries by default (Gatekeeper)
$SUDO xattr -dr com.apple.quarantine "${INSTALLATION_DIR}" 2>/dev/null || true

# ----- Validate -----
if ! command -v "${BIN_NAME}" >/dev/null 2>&1; then
	error "Binary not found in PATH"
	exit 1
fi

finish "Installation OK"

# ----- Man pages -----
install_man_pages

# ----- LICENSE -----
install_license

# ----- Git -----
check_git

finish "Done!"
