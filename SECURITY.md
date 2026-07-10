# Verifying Binary Integrity

For security-conscious users, we provide cryptographic signatures for all releases. This ensures that the binaries you download haven't been tampered with.

## Prerequisites

* GPG (GNU Privacy Guard) installed on your system
* The project's public GPG key imported in your system's keyring

## Importing the Public Key

To verify signatures, you first need to import the project's public key:

```sh
# Import the public key from a keyserver (e.g., keys.openpgp.org)
gpg --keyserver keys.openpgp.org --recv-keys E6A5CC75350F3DCE

# Or, download our public key at https://github.com/orbitbits/pubkey/blob/main/pubkey.asc
gpg --import pubkey.asc
```

## Verifying the Binary

After downloading the binary and its signature file (`.asc`), verify the integrity:

```sh
# Download the binary and signature
# Example: roost-0.1.0-x86_64 and SHA256SUMS.asc

# Verify the SHA256SUMS signature
gpg --verify SHA256SUMS.asc

# If valid, verify the binary against the checksums
sha256sum -c SHA256SUMS

# Both commands should show "Good signature" and "OK"
```

## What This Ensures

* **Authenticity**: The release was created by project maintainers (verified by GPG signature)
* **Integrity**: The binary hasn't been modified or corrupted (verified by SHA256 checksum)
* **Non-Repudiation**: The maintainers can be held accountable for releases
