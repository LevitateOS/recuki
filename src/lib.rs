//! recuki - Unified Kernel Image (UKI) builder.
//!
//! Builds UKIs using `ukify` from systemd. A UKI combines kernel + initramfs +
//! cmdline into one PE binary, so Secure Boot has a single file to sign (recuki
//! itself does not sign).

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct UkiConfig {
    /// Path to the kernel image (vmlinuz).
    pub kernel: PathBuf,
    pub initrd: PathBuf,
    pub cmdline: String,
    /// Output path for the .efi file.
    pub output: PathBuf,
    /// Optional OS branding for boot menu display.
    pub os_release: Option<OsRelease>,
}

/// OS release information for UKI branding.
#[derive(Debug, Clone)]
pub struct OsRelease {
    /// OS name (e.g., "LevitateOS").
    pub name: String,
    /// OS identifier (e.g., "levitateos").
    pub id: String,
    /// OS version (e.g., "1.0").
    pub version: String,
}

impl UkiConfig {
    pub fn new(
        kernel: impl Into<PathBuf>,
        initrd: impl Into<PathBuf>,
        cmdline: impl Into<String>,
        output: impl Into<PathBuf>,
    ) -> Self {
        Self {
            kernel: kernel.into(),
            initrd: initrd.into(),
            cmdline: cmdline.into(),
            output: output.into(),
            os_release: None,
        }
    }
}

impl OsRelease {
    pub fn new(name: impl Into<String>, id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            version: version.into(),
        }
    }

    /// Generate os-release file content.
    pub fn to_content(&self) -> String {
        format!(
            "NAME=\"{}\"\n\
             ID={}\n\
             VERSION=\"{}\"\n\
             PRETTY_NAME=\"{} {}\"\n",
            self.name, self.id, self.version, self.name, self.version
        )
    }
}

/// Uses `ukify` from systemd. Requires systemd-ukify to be installed.
///
/// # Errors
///
/// Returns an error if:
/// - `ukify` is not installed
/// - The kernel or initrd files don't exist
/// - The output directory cannot be created
/// - `ukify` fails to build the UKI
pub fn build_uki(config: &UkiConfig) -> Result<()> {
    if !config.kernel.exists() {
        bail!("Kernel not found: {}", config.kernel.display());
    }
    if !config.initrd.exists() {
        bail!("Initrd not found: {}", config.initrd.display());
    }

    if let Some(parent) = config.output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }
    }

    // ukify reads the cmdline and os-release from files (`@path`).
    let scratch = tempfile::tempdir().context("creating ukify scratch directory")?;
    let cmdline_file = scratch.path().join("cmdline");
    std::fs::write(&cmdline_file, &config.cmdline)
        .with_context(|| format!("Failed to write cmdline file: {}", cmdline_file.display()))?;

    let mut cmd = Command::new("ukify");
    cmd.arg("build")
        .args(["--linux", &config.kernel.to_string_lossy()])
        .args(["--initrd", &config.initrd.to_string_lossy()])
        .args(["--cmdline", &format!("@{}", cmdline_file.display())])
        .args(["--output", &config.output.to_string_lossy()]);

    if let Some(ref os_release) = config.os_release {
        let path = scratch.path().join("os-release");
        std::fs::write(&path, os_release.to_content())
            .with_context(|| format!("Failed to write os-release file: {}", path.display()))?;
        cmd.args(["--os-release", &format!("@{}", path.display())]);
    }

    let output = cmd
        .output()
        .context("Failed to execute 'ukify' (systemd's UKI builder); is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "ukify failed (exit code {}):\n{}",
            output.status.code().unwrap_or(-1),
            stderr
        );
    }

    Ok(())
}
