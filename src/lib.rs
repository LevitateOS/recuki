//! recuki - Unified Kernel Image (UKI) builder.
//!
//! Builds UKIs using `ukify` from systemd. UKIs combine kernel + initramfs + cmdline
//! into a single signed PE binary for simplified boot and Secure Boot support.
//!
//! # Example
//!
//! ```ignore
//! use recuki::{UkiConfig, build_uki};
//!
//! let config = UkiConfig::new("vmlinuz", "initramfs.img", "root=LABEL=root rw", "boot.efi");
//! build_uki(&config)?;
//! ```

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for building a UKI.
#[derive(Debug, Clone)]
pub struct UkiConfig {
    /// Path to the kernel image (vmlinuz).
    pub kernel: PathBuf,
    /// Path to the initramfs image.
    pub initrd: PathBuf,
    /// Kernel command line string.
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
    /// Create a new UKI configuration.
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

    /// Set OS release branding.
    pub fn with_os_release(mut self, name: &str, id: &str, version: &str) -> Self {
        self.os_release = Some(OsRelease {
            name: name.to_string(),
            id: id.to_string(),
            version: version.to_string(),
        });
        self
    }
}

impl OsRelease {
    /// Create a new OS release configuration.
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

/// Build a UKI from the given configuration.
///
/// Uses `ukify` from systemd. Requires systemd-ukify to be installed.
///
/// # Errors
///
/// Returns an error if:
/// - `ukify` is not installed
/// - The kernel or initrd files don't exist
/// - The output directory doesn't exist
/// - `ukify` fails to build the UKI
pub fn build_uki(config: &UkiConfig) -> Result<()> {
    // Validate inputs exist
    if !config.kernel.exists() {
        bail!("Kernel not found: {}", config.kernel.display());
    }
    if !config.initrd.exists() {
        bail!("Initrd not found: {}", config.initrd.display());
    }

    // Ensure output directory exists
    if let Some(parent) = config.output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }
    }

    // Write cmdline to temp file (ukify reads from file with @ prefix)
    let cmdline_file = config.output.with_extension("cmdline");
    std::fs::write(&cmdline_file, &config.cmdline)
        .with_context(|| format!("Failed to write cmdline file: {}", cmdline_file.display()))?;

    // Build ukify command
    let mut cmd = Command::new("ukify");
    cmd.arg("build")
        .args(["--linux", &config.kernel.to_string_lossy()])
        .args(["--initrd", &config.initrd.to_string_lossy()])
        .args(["--cmdline", &format!("@{}", cmdline_file.display())])
        .args(["--output", &config.output.to_string_lossy()]);

    // Write os-release if provided
    let os_release_file = if let Some(ref os_release) = config.os_release {
        let path = config.output.with_extension("os-release");
        std::fs::write(&path, os_release.to_content())
            .with_context(|| format!("Failed to write os-release file: {}", path.display()))?;
        cmd.args(["--os-release", &format!("@{}", path.display())]);
        Some(path)
    } else {
        None
    };

    // Run ukify
    let output = cmd.output().context(
        "Failed to execute 'ukify'. Is systemd-ukify installed?\n\
         Install with: sudo dnf install systemd-ukify",
    )?;

    // Cleanup temp files regardless of result
    let _ = std::fs::remove_file(&cmdline_file);
    if let Some(path) = os_release_file {
        let _ = std::fs::remove_file(path);
    }

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

/// Build a UKI with minimal arguments.
///
/// Convenience wrapper around [`build_uki`] for simple cases.
pub fn build_uki_simple(kernel: &Path, initrd: &Path, cmdline: &str, output: &Path) -> Result<()> {
    let config = UkiConfig::new(kernel, initrd, cmdline, output);
    build_uki(&config)
}

/// Build a UKI with OS branding.
///
/// Convenience wrapper that includes OS release information.
pub fn build_uki_branded(
    kernel: &Path,
    initrd: &Path,
    cmdline: &str,
    output: &Path,
    os_name: &str,
    os_id: &str,
    os_version: &str,
) -> Result<()> {
    let config =
        UkiConfig::new(kernel, initrd, cmdline, output).with_os_release(os_name, os_id, os_version);
    build_uki(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_release_content() {
        let os = OsRelease::new("LevitateOS", "levitateos", "1.0");
        let content = os.to_content();

        assert!(content.contains("NAME=\"LevitateOS\""));
        assert!(content.contains("ID=levitateos"));
        assert!(content.contains("VERSION=\"1.0\""));
        assert!(content.contains("PRETTY_NAME=\"LevitateOS 1.0\""));
    }

    #[test]
    fn test_uki_config_builder() {
        let config = UkiConfig::new("vmlinuz", "initramfs.img", "root=/dev/sda1", "boot.efi")
            .with_os_release("Test", "test", "1.0");

        assert_eq!(config.kernel, PathBuf::from("vmlinuz"));
        assert_eq!(config.initrd, PathBuf::from("initramfs.img"));
        assert_eq!(config.cmdline, "root=/dev/sda1");
        assert_eq!(config.output, PathBuf::from("boot.efi"));
        assert!(config.os_release.is_some());
    }

    #[test]
    fn test_build_uki_validates_kernel() {
        let config = UkiConfig::new(
            "/nonexistent/vmlinuz",
            "/nonexistent/initramfs.img",
            "root=/dev/sda1",
            "boot.efi",
        );

        let err = build_uki(&config).unwrap_err();
        assert!(err.to_string().contains("Kernel not found"));
    }
}
