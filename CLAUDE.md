# CLAUDE.md - recuki

## What is recuki?

Standalone UKI (Unified Kernel Image) builder. Wraps `ukify` from systemd with a clean CLI.

UKIs combine kernel + initramfs + cmdline into a single PE binary. This simplifies boot and enables Secure Boot with just one file to sign.

## What Belongs Here

- UKI building logic
- OS-release branding support
- ukify wrapper

## What Does NOT Belong Here

| Don't put here | Put it in |
|----------------|-----------|
| LevitateOS-specific UKI entries | `leviso/src/artifact/uki.rs` |
| Boot entry constants | `distro-spec/` |
| ISO building | `leviso/` |

## Commands

```bash
cargo build --release
cargo check # Unit tests are prohibited; use focused integration/runtime checks.
```

## Usage

```bash
# Basic UKI
recuki -k vmlinuz -i initramfs.img -c 'root=LABEL=root rw' -o boot.efi

# With OS branding (shows in boot menu)
recuki -k vmlinuz -i initramfs.img -c 'root=LABEL=root rw' \
       --os-name 'LevitateOS' --os-id levitateos --os-version '1.0' \
       -o levitateos.efi
```

## Library Usage

```rust
use recuki::{UkiConfig, build_uki};

let config = UkiConfig::new("vmlinuz", "initramfs.img", "root=LABEL=root rw", "boot.efi")
    .with_os_release("LevitateOS", "levitateos", "1.0");
build_uki(&config)?;
```

## Requirements

Requires `ukify` from systemd-ukify:
```bash
sudo dnf install systemd-ukify
```
