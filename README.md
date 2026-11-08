# recuki

Library that builds Unified Kernel Images (UKI) from kernel + initramfs + cmdline with `ukify` from systemd. Used by `reciso` and `distro-builder`; there is no command-line binary.

A UKI combines kernel, initramfs and boot parameters into a single UEFI PE binary, so Secure Boot has one file to sign (recuki does not sign).
