# recuki

Build Unified Kernel Images (UKI) from kernel + initramfs + cmdline.

UKIs combine kernel, initramfs, and boot parameters into a single UEFI PE binary. This simplifies boot and enables Secure Boot with just one file to sign.

## Status

**Beta.** Works for standard systemd-boot setups.

## Usage

```bash
# Basic UKI
recuki -k vmlinuz -i initramfs.img -c 'root=LABEL=root rw' -o boot.efi

# With OS branding (shows in boot menu)
recuki -k vmlinuz -i initramfs.img -c 'root=LABEL=root rw' \
       --os-name 'LevitateOS' --os-id levitateos --os-version '1.0' \
       -o levitateos.efi
```

## Options

```
recuki [OPTIONS] -k <KERNEL> -i <INITRD> -c <CMDLINE> -o <OUTPUT>

-k, --kernel <PATH>      Path to kernel (vmlinuz)
-i, --initrd <PATH>      Path to initramfs
-c, --cmdline <STRING>   Kernel command line
-o, --output <PATH>      Output UKI path (.efi)

--os-name <NAME>         OS name for boot menu
--os-id <ID>             OS ID (lowercase, no spaces)
--os-version <VERSION>   OS version string
```

## Library Usage

```rust
use recuki::{UkiConfig, build_uki};

let config = UkiConfig::new("vmlinuz", "initramfs.img", "root=LABEL=root rw", "boot.efi")
    .with_os_release("LevitateOS", "levitateos", "1.0");
build_uki(&config)?;
```

## What It Does

1. Creates a temporary os-release file (if branding specified)
2. Calls `ukify build` with kernel, initrd, cmdline
3. Outputs a single .efi file

## What It Does NOT Do

- Sign the UKI (use `sbsign` separately)
- Install to ESP (copy it yourself)
- Generate initramfs (use `recinit`)

## Requirements

- systemd-ukify: `sudo dnf install systemd-ukify`

## Building

```bash
cargo build --release
```

## License

MIT OR Apache-2.0
