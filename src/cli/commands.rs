//! Explicit CLI subcommands.

use clap::{Args, Subcommand};

/// Top-level command variants.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Interactive setup for provider/model/API key.
    Setup,
    /// Run doctor checks.
    Doctor,
    /// Inspect storage state.
    InspectStorage,
    /// Inspect a systemd service.
    ServiceStatus(ServiceStatusArgs),
    /// Inspect package installation and version state.
    PackageStatus(PackageStatusArgs),
    /// Install components.
    Install {
        /// Installation subcommand.
        #[command(subcommand)]
        command: InstallCommands,
    },
    /// Troubleshoot firewall connectivity.
    TroubleshootFirewall(FirewallArgs),
    /// Diagnose reboot issues.
    Diagnose {
        /// Diagnose subcommand.
        #[command(subcommand)]
        command: DiagnoseCommands,
    },
    /// Inspect recent system or service logs.
    Logs(LogsArgs),
    /// Show maintenance best-practice guidance.
    Maintenance,
    /// Plan root resize actions.
    ResizeRoot(ResizeRootArgs),
}

/// Args for `service-status`.
#[derive(Debug, Args)]
pub struct ServiceStatusArgs {
    /// Service name.
    pub name: String,
}

/// Args for `package-status`.
#[derive(Debug, Args)]
pub struct PackageStatusArgs {
    /// Package name.
    pub name: String,
}

/// Args for `troubleshoot-firewall`.
#[derive(Debug, Args)]
pub struct FirewallArgs {
    /// TCP port to inspect.
    #[arg(long)]
    pub port: Option<u16>,
}

/// Args for `logs`.
#[derive(Debug, Args)]
pub struct LogsArgs {
    /// Optional service name to focus on.
    #[arg(long)]
    pub service: Option<String>,
}

/// Args for `resize-root`.
#[derive(Debug, Args)]
pub struct ResizeRootArgs {
    /// Desired target size (advisory only).
    #[arg(long)]
    pub target_size: Option<String>,
}

/// Install group commands.
#[derive(Debug, Subcommand)]
pub enum InstallCommands {
    /// Install an Ubuntu package via apt.
    Package {
        /// Package name to install.
        name: String,
    },
    /// Install Tomcat via apt.
    Tomcat,
}

/// Diagnose group commands.
#[derive(Debug, Subcommand)]
pub enum DiagnoseCommands {
    /// Diagnose reboot or kernel issues.
    Reboot,
}
