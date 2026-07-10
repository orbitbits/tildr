---
title: TILDR-SECURITY
section: 1
header: User Commands
footer: Tildr
date: 2026
---

# NAME

tildr-security — binary verification and release security for Tildr

# SYNOPSIS

```sh
gpg --verify SHA256SUMS.asc
sha256sum -c SHA256SUMS
```

# DESCRIPTION

All Tildr releases are signed with a GPG key held by the project maintainers.
Verifying the signature before installation ensures the binary has not been tampered with.

# PREREQUISITES

- GPG (GNU Privacy Guard) installed on your system
- The project's public GPG key imported into your keyring

# IMPORTING THE PUBLIC KEY

Import the maintainer's public key from a keyserver:

```sh
gpg --keyserver keys.openpgp.org --recv-keys E6A5CC75350F3DCE
```

Or download and import it directly:

```sh
curl -fsSL https://raw.githubusercontent.com/orbitbits/pubkey/main/pubkey.asc | gpg --import
```

# VERIFYING A RELEASE

After downloading the binary, **SHA256SUMS**, and **SHA256SUMS.asc**:

```sh
# Step 1: verify the GPG signature over SHA256SUMS
gpg --verify SHA256SUMS.asc

# Step 2: verify the binary checksum
sha256sum -c SHA256SUMS
```

Both commands must succeed. A valid run looks like:

```
gpg: Good signature from "William Canin <hello.williamcanin@gmail.com>"
tildr-0.1.0-linux-x86_64: OK
```

# WHAT THIS ENSURES

**Authenticity**
:   The release was produced by the project maintainers, verified by GPG signature.

**Integrity**
:   The binary has not been modified or corrupted since it was signed, verified by SHA256 checksum.

**Non-repudiation**
:   The maintainers can be held accountable for the content of each release.

# FILES

*SHA256SUMS*
:   Checksums file listing the SHA256 hash of each release artifact.

*SHA256SUMS.asc*
:   Detached GPG signature over the SHA256SUMS file.

# SEE ALSO

**tildr(1)**, **tildr-config(1)**, **tildr-commands(1)**, **tildr-plugins(1)**
