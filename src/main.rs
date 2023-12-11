use colored::*;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::env;
use clap::{CommandFactory, Parser, Subcommand};
use color_backtrace::install;
use log::{error, info};
use crate::DebugLevel::Verbose;


#[tokio::main]
async fn main() {
    env_logger::init();
    install();


    let cli = CargoHelper::parse();

    match &cli.command {
        Commands::GenerateFigSpec => clap_complete::generate(
            clap_complete_fig::Fig,
            &mut CargoHelper::command(),
            "cargo_migration",
            &mut std::io::stdout(),
        ),
        Commands::CargoMigrationCmd => {
            let home_dir = env::var("HOME").unwrap_or_else(|_| {
                error!(
            "{} Unable to determine user's home directory.",
            "Error:".bright_red()
        );
                std::process::exit(1);
            });


            let default_uninstall_script_file = format!("{}/Downloads/uninstall_packages.sh", home_dir);
            let default_install_script_file = format!("{}/Downloads/install_packages.sh", home_dir);

            let uninstall_script_file = env::var("UNINSTALL_SCRIPT_FILE")
                .unwrap_or_else(|_| default_uninstall_script_file.clone());

            let install_script_file = env::var("INSTALL_SCRIPT_FILE")
                .unwrap_or_else(|_| default_install_script_file.clone());

            let mut uninstall_file = File::create(&uninstall_script_file)
                .await
                .expect(&format!("{} Error creating file.", "Error:".bright_red()));

            let mut install_file = File::create(&install_script_file)
                .await
                .expect(&format!("{} Error creating file.", "Error:".bright_red()));

            // Run for Homebrew
            generate_scripts::<Homebrew>(&mut uninstall_file, &mut install_file).await;

            info!(
        "{} Uninstall script generated at: {}",
        "Success:".bright_green(),
        uninstall_script_file
    );
            info!(
        "{} Install script generated at: {}",
        "Success:".bright_green(),
        install_script_file
    );
        }
        _ => {
            error!(
                "{} Command not found.",
                "Error:".bright_red()
            );
            std::process::exit(1);
        }
    }
}


#[async_trait::async_trait]
trait PackageManager {
    fn new() -> Self;
    async fn get_installed_packages(&self) -> Vec<String>;
    async fn has_rust_dependency(&self, package: &str) -> bool;
    async fn get_version(&self, package: &str) -> Option<String>;
    fn uninstall_command(&self, package: &str) -> String;
    fn install_command(&self, package: &str) -> String;
}


struct Homebrew;

#[async_trait::async_trait]
impl PackageManager for Homebrew {
    fn new() -> Self {
        Self
    }

    async fn get_installed_packages(&self) -> Vec<String> {
        run_command("brew", &["list", "--formula"]).await
    }

    async fn has_rust_dependency(&self, package: &str) -> bool {
        run_command_contains("brew", &["info", package], "rust").await
    }

    async fn get_version(&self, package: &str) -> Option<String> {
        let output = run_command("brew", &["info", "--json=v1", package]).await;
        let info: serde_json::Value =
            serde_json::from_str(&output.join("\n")).expect("Error parsing JSON");

        let version = info[0]["versions"]["stable"]
            .as_str()
            .map(String::from);

        version
    }

    fn uninstall_command(&self, package: &str) -> String {
        format!("brew uninstall {}\n", package)
    }

    fn install_command(&self, package: &str) -> String {
        format!("cargo install {}\n", package)
    }
}
// ... (Cargo implementation remains the same)

struct Cargo;

#[async_trait::async_trait]
impl PackageManager for Cargo {
    fn new() -> Self {
        Self
    }

    async fn get_installed_packages(&self) -> Vec<String> {
        Vec::new()
    }

    async fn has_rust_dependency(&self, _package: &str) -> bool {
        true
    }

    async fn get_version(&self, package: &str) -> Option<String> {
        let output = run_command("cargo", &["search", "--limit", "1", "--offline", package]).await;

        if output.is_empty() {
            return None;
        }

        let cargo_output = output.join("\n");
        cargo_output
            .lines()
            .next()?
            .split_whitespace()
            .last()
            .map(String::from)
    }

    fn uninstall_command(&self, package: &str) -> String {
        format!("cargo uninstall {}", package)
    }

    fn install_command(&self, package: &str) -> String {
        format!("cargo install {}", package)
    }
}

pub fn install_command_cargo(package: &str) -> String {
    format!("cargo install {}\n", package)
}


async fn run_command(command: &str, args: &[&str]) -> Vec<String> {
    let output = tokio::process::Command::new(command)
        .args(args)
        .output()
        .await
        .expect(&format!("{} Error running {} command", "Error:".bright_red(), command));

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect()
}

async fn run_command_contains(command: &str, args: &[&str], pattern: &str) -> bool {
    let output = run_command(command, args).await;
    output.iter().any(|line| line.contains(pattern))
}

async fn generate_scripts<T: PackageManager>(
    uninstall_file: &mut File,
    install_file: &mut File,
) {
    let package_manager = T::new();
    let packages = package_manager.get_installed_packages().await;

    for package in packages {
        if package_manager.has_rust_dependency(&package).await {
            if let Some(version) = package_manager.get_version(&package).await {
                uninstall_file
                    .write_all(package_manager.uninstall_command(&package).as_bytes())
                    .await
                    .expect(&format!(
                        "{} Error writing to uninstall script.",
                        "Error:".bright_red()
                    ));

                println!(
                    "{} Uninstalling {}...",
                    "Info:".bright_cyan(),
                    &package
                );

                install_file
                    .write_all(install_command_cargo(&format!("{}@{}", &package, version)).as_bytes())
                    .await
                    .expect(&format!(
                        "{} Error writing to install script.",
                        "Error:".bright_red()
                    ));

                println!(
                    "{} Installing {}...",
                    "Info:".bright_cyan(),
                    &package
                );
            }
        }
    }

    println!(
        "{} Installing packages from Homebrew...",
        "Info:".bright_cyan()
    );
}


#[derive(clap::Parser)]
struct CargoHelper {
    #[clap(subcommand)]
    command: Commands,

}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct CargoMigrationArgs {
    #[arg(short, long)]
    uninstall_script_file_location: Option<String>,
    #[arg(short, long)]
    install_script_file_location: Option<String>,

    #[arg(short, long)]
    config_file_location: Option<String>,

    // TODO ADD DEFAULT PACKAGE MANAGER
    #[arg(short, long)]
    package_manager: Option<String>,

    #[arg(short, long)]
    offline: bool,

    #[arg(short, long)]
    run: RunMode,

    #[arg(short, long)]
    debug: DebugLevel,

}

// #[derive(Subcommand)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[derive(clap::Parser)]
enum RunMode {
    Automatic,
    Interactive,
    DryRun,
}

impl From<String> for RunMode {
    fn from(value: String) -> Self {
        if value.trim().to_lowercase().contains("auto") {
            return Self::Automatic;
        } else if value.trim().to_lowercase().contains("int") {
            return Self::Interactive;
        } else {
            return Self::DryRun;
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[derive(clap::Parser)]
enum PackageManagerType {
    Homebrew,
    Cargo,
    Apt,
    Winget,
    // Ports,
    // Pkg,
    // Choco,
    /*    Npm,
        Pip,
        Go,
        Dnf,
        Nix,
        Scoop,
        Pacman,
        Dpkg,
        Flatpak,
        Snap,
        Other,*/
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum DebugLevel {
    Verbose,
    Quiet,
    Normal,
}

impl From<String> for DebugLevel {
    fn from(value: String) -> Self {
        if value.trim().to_lowercase().contains("ver") {
            return Self::Verbose;
        } else if value.trim().to_lowercase().contains("quiet") {
            return Self::Quiet;
        } else {
            return Self::Normal;
        }


        // todo!("fix this to not have to match user string and just take the flag in order for the arg")
    }
}


#[derive(Subcommand)]
enum Commands {
    CargoMigrationCmd,
    GenerateFigSpec,
}


// todo refactor into crates
// todo add tests
// todo add readme
// todo add docs
// todo add fig shell completions, and inshellisense zsh, bash, fish, powershell, elvish, xonsh, ion, etc
// todo reduce file size
// todo add a dry run command
// todo add more implementations for other package managers
// todo add command line interface
// todo add backup for config files
// todo add implementations to move configs and data to cargo install dir
// todo check existing cargo packages to see if they are installed both cargo and the other package managers
// todo add automation of uninstalling and installing
// todo improve speed
// todo add ability to work offline
// todo add ability to work with multiple package managers
// todo speed improvements
// todo add cache for existing packages
// todo add an api for package managers
// todo add implementation for apt, winget, ports, dnf, choco,
// todo then  npm, go, pip,  then  nix, pacman, scoop, than last pacman, dpkg, pkg, flatpak, snap etc
// todo add suggestion for similar cargo package in other package managers
// todo be harsher on finding out if a package is a cargo package
// todo add ability to use latest version or same version as package manager
