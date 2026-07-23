---
layout: doc
part: 8
section: Encryption
menu: tildr
version: "0.3.1"
doc_product: tildr
logo: https://raw.githubusercontent.com/orbitbits/tildr/refs/heads/main/.github/brand/logo-text/compact/tildr-variation-3.svg
title: Secret Management
description: Manage your HOME files and directories with symlinks and Git.
date: 2026-04-18 17:59:04 -0300
tags: [Rust, CLI, Declarative, Dotfiles, Synchronization, Reproducible]
permalink: /tildr/documentation/0.3.1/secret-management/
---

<!-- markdownlint-disable MD024 -->

## `tildr secret`

Manages encryption of sensitive files in your dotfiles repository using GPG encryption.

Some files you manage with Tildr — such as SSH keys, GPG private keys, or any file containing credentials — should never be stored in plain text in a repository, especially a public one. `tildr secret` solves this by encrypting those files into a single encrypted bundle that is safe to commit and push.

### How it works

Tildr maintains two files at the root of your repository:

* `.tildr/encrypted-items` — a plaintext manifest listing the relative paths of all registered sensitive files, one per line. This file is committed to the repository.
* `.tildr/encrypted.gpg` — an encrypted bundle containing all the registered files packed together. This file is also committed to the repository.

Sensitive files managed through symlinks may physically live under `common/` or `profiles/<name>/`, but they are **never kept in Git tracking**. When you register a file with `tildr secret add`, Tildr adds the effective physical source path to `.gitignore` and removes that source from Git tracking if necessary. Only the encrypted bundle enters version control.

### Encryption model

Tildr supports two GPG encryption modes, configured via `[crypto].mode` in `config.toml`:

**Symmetric** (`mode = "symmetric"`, default):

* No key pair required — only a passphrase
* GPG prompts for the passphrase via the system pinentry on first use
* The same passphrase must be used to decrypt on any machine
* Simpler setup, suitable for single-user environments

**Asymmetric** (`mode = "asymmetric"`):

* Uses an existing GPG key pair — no separate passphrase to remember
* `[crypto].gpg_key` must be set to the recipient key ID or email, or Tildr prompts interactively on first use and saves the choice automatically
* Decryption uses the private key silently (subject to GPG Agent caching)
* Preferred when you already manage GPG keys and want a seamless new-machine setup

In both modes:

* GPG must be installed on the system (`gpg` in `PATH`)
* Tildr creates a tar archive of all registered files and encrypts it as a single `.tildr/encrypted.gpg` bundle
* Files are archived with **relative paths** so they extract correctly to any `$HOME` regardless of username or machine
* Decryption is always automatic — GPG detects the encryption type from the bundle

### Subcommands

```sh
tildr secret add <FILE>
tildr secret rm <FILE>
tildr secret list
tildr secret encrypt
tildr secret decrypt
```

---

#### `tildr secret add`

Registers a sensitive file, adds it to `.gitignore`, removes it from Git tracking if necessary, and re-encrypts the full bundle.

```sh
tildr secret add ~/.ssh/id_rsa
tildr secret add ~/.gnupg/private-keys-v1.d/ABC123.key
```

Behavior:

* The file must exist in `$HOME`
* The relative path is added to `.tildr/encrypted-items`
* The effective profile source path, such as `common/.ssh/id_rsa` or `profiles/linux/.ssh/id_rsa`, is appended to the root `.gitignore` so it is never committed
* If the file was already tracked by Git, `git rm --cached` is run to remove it from the index without deleting the file from disk
* All registered files (including the newly added one) are re-packed into a tar archive and re-encrypted into `.tildr/encrypted.gpg`
* GPG will prompt for a passphrase (symmetric) or use the configured key (asymmetric) on encryption
* Auto-commits the repository when `git.auto_commit = true`

> **Important:** the file registered with `tildr secret add` must be from `$HOME`, not from the repository. The original file lives in `$HOME` and only the encrypted bundle lives in the repository.

---

#### `tildr secret rm`

  Remove a file from the encrypted bundle.

  ```sh
  tildr secret rm .ssh/id_rsa
  tildr secret rm ~/.ssh/id_rsa
  tildr secret rm $HOME/.ssh/id_rsa
```

Behavior:

* The relative path is removed from `.tildr/encrypted-items`
* If other files remain registered, the bundle is re-encrypted without the removed entry
* If no files remain, `.tildr/encrypted.gpg` is deleted from the repository
* The original file in `$HOME` is not touched
* Auto-commits the repository when `git.auto_commit = true`

---

#### `tildr secret list`

Lists all sensitive files currently registered in `.tildr/encrypted-items`.

```sh
tildr secret list
```

Output example:

```text
Sensitive files
---------------
  .ssh/id_rsa
  .ssh/id_rsa.pub
  .gnupg/private-keys-v1.d/ABC123.key
```

---

#### `tildr secret encrypt`

Manually re-encrypts all registered files into the bundle using their current content from `$HOME`.

```sh
tildr secret encrypt
```

Behavior:

* Reads all entries from `.tildr/encrypted-items`
* Packs the current content of each file from `$HOME` into a tar archive
* Encrypts the archive into `.tildr/encrypted.gpg`, replacing the previous bundle
* In symmetric mode, GPG may prompt for a passphrase depending on the agent cache state
* In asymmetric mode, GPG uses the configured key silently (subject to GPG Agent caching)
* Auto-commits the repository when `git.auto_commit = true`

Use this command after editing a registered sensitive file when you want to update the bundle manually. If you use `tildr sync`, re-encryption happens automatically before the push — so in a typical workflow, running `tildr secret encrypt` explicitly is optional.

---

#### `tildr secret decrypt`

Decrypts the bundle and restores all registered files to their original locations in `$HOME`.

```sh
tildr secret decrypt
```

Behavior:

* Decrypts `.tildr/encrypted.gpg` using GPG — the encryption type (symmetric or asymmetric) is detected automatically from the bundle
* Extracts the tar archive into `$HOME`, restoring each file to its registered path
* In symmetric mode, GPG prompts for the passphrase via the system pinentry
* In asymmetric mode, GPG uses the private key silently (subject to GPG Agent caching)
* Files are extracted with relative paths — they always land correctly regardless of username

Use this command when you need to restore sensitive files manually, for example after running `tildr secret rm` or after setting up a new machine without going through `tildr import`.

---

### Integration with `tildr sync`

When `tildr sync` is about to push commits to the remote, it automatically re-encrypts all registered files before the push. This ensures the bundle in the remote repository always reflects the current state of your sensitive files, even if you edited them since the last encryption.

Re-encryption only happens on push scenarios (`PushOnly` and `Diverged`). Pull-only and up-to-date scenarios do not trigger re-encryption.

---

### Integration with `tildr import`

When `tildr import` is run and the cloned repository contains a `.tildr/encrypted.gpg` bundle, Tildr automatically decrypts it after applying symlinks.

* In symmetric mode, GPG prompts for the passphrase via the system pinentry
* In asymmetric mode, GPG uses the private key — which must already be available in the keyring at import time (e.g. restored by `tildr apply` if you manage `~/.gnupg` with Tildr)

If GPG is not installed at import time, Tildr warns but does not fail. You can decrypt later manually with `tildr secret decrypt` once GPG is available.

---

### GPG Agent and passphrase caching

GPG uses a background agent (`gpg-agent`) that caches credentials in memory for a period of time after the first successful use.

In **symmetric mode**, the agent caches the passphrase — after you enter it once, subsequent GPG operations including `tildr secret encrypt`, `tildr secret decrypt`, and the automatic re-encryption in `tildr sync` may not prompt for the passphrase again until the cache expires.

In **asymmetric mode**, the agent caches the private key passphrase if the key has one. Operations proceed silently while the cache is active.

Default cache duration is **600 seconds (10 minutes)**. You can configure it in `~/.gnupg/gpg-agent.conf`:

```text
default-cache-ttl 600
max-cache-ttl 7200
```

To force GPG to forget the cached passphrase immediately:

```sh
gpg-connect-agent reloadagent /bye
```

This behavior is managed entirely by GPG and the system pinentry — Tildr has no control over passphrase caching.

---

### Inspecting the bundle

To list the files inside the bundle without decrypting them to disk:

```sh
gpg --decrypt .tildr/encrypted.gpg 2>/dev/null | tar tv
```

This is useful to verify the bundle contents are correct before pushing to a remote repository.

---

### Typical secret management workflow

```sh
# Register sensitive files
tildr secret add ~/.ssh/id_rsa
tildr secret add ~/.ssh/id_rsa.pub

# Verify what is registered
tildr secret list

# Edit a registered file, then update the bundle
tildr secret encrypt

# Push everything including the updated bundle
tildr sync

# On a new machine after tildr import
tildr secret decrypt
```
