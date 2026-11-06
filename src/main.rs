//! recuki - Build Unified Kernel Images (UKI).
//!
//! UKIs combine kernel + initramfs + cmdline into a single PE binary.
//! This simplifies boot and enables Secure Boot with a single file to sign.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use recuki::{build_uki, OsRelease, UkiConfig};

#[derive(Parser, Debug)]
#[command(
    name = "recuki",
    about = "Build Unified Kernel Images (UKI) from kernel + initramfs + cmdline",
    version,
    after_help = "\
EXAMPLES:
    # Basic UKI
    recuki -k vmlinuz -i initramfs.img -c 'root=LABEL=root rw' -o boot.efi

    # With OS branding (shows in boot menu)
    recuki -k vmlinuz -i initramfs.img -c 'root=LABEL=root rw' \\
           --os-name 'LevitateOS' --os-id levitateos --os-version '1.0' \\
           -o levitateos.efi

REQUIREMENTS:
    Requires 'ukify' from systemd.
"
)]
struct Args {
    /// Path to kernel image (vmlinuz)
    #[arg(short = 'k', long = "kernel")]
    kernel: PathBuf,

    /// Path to initramfs image
    #[arg(short = 'i', long = "initrd")]
    initrd: PathBuf,

    /// Kernel command line
    #[arg(short = 'c', long = "cmdline")]
    cmdline: String,

    /// Output path for the .efi file
    #[arg(short = 'o', long = "output")]
    output: PathBuf,

    /// OS name for boot menu display (e.g., "LevitateOS")
    #[arg(long = "os-name", requires_all = ["os_id", "os_version"])]
    os_name: Option<String>,

    /// OS identifier (e.g., "levitateos")
    #[arg(long = "os-id", requires_all = ["os_name", "os_version"])]
    os_id: Option<String>,

    /// OS version (e.g., "1.0")
    #[arg(long = "os-version", requires_all = ["os_name", "os_id"])]
    os_version: Option<String>,

    /// Quiet mode - only print errors
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = UkiConfig::new(&args.kernel, &args.initrd, &args.cmdline, &args.output);

    if let (Some(name), Some(id), Some(version)) = (&args.os_name, &args.os_id, &args.os_version) {
        config.os_release = Some(OsRelease::new(name, id, version));
    }

    if !args.quiet {
        println!("Building UKI: {}", args.output.display());
        println!("  Kernel: {}", args.kernel.display());
        println!("  Initrd: {}", args.initrd.display());
        if let Some(ref os) = config.os_release {
            println!("  OS: {} {} ({})", os.name, os.version, os.id);
        }
    }

    build_uki(&config)?;

    if !args.quiet {
        println!("Done.");
    }

    Ok(())
}
