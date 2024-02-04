use color_backtrace::install;
use colored::*;
use log::{error, info};
use std::env;
use std::fs::File;
use std::io::Write;

use brew_man::Homebrew;
use packmanager::PackageManager;
use rayon::prelude::*;

mod packmanager;
mod cargo_man;
mod apt_man;
mod win_man;
mod brew_man;


fn main() {
    env_logger::init();
    install();

    let home_dir = env::var("HOME").unwrap_or_else(|_| {
        error!(
            "{} Unable to determine user's home directory.",
            "Error:".bright_red()
        );
        std::process::exit(1);
    });

    let default_uninstall_script_file = format!("{}/Downloads/uninstall_packages.sh", home_dir);
    let default_install_script_file = format!("{}/Downloads/install_packages.sh", home_dir);

    let uninstall_script_file =
        env::var("UNINSTALL_SCRIPT_FILE").unwrap_or_else(|_| default_uninstall_script_file.clone());

    let install_script_file =
        env::var("INSTALL_SCRIPT_FILE").unwrap_or_else(|_| default_install_script_file.clone());

    let mut uninstall_file = File::create(&uninstall_script_file)

        .expect(&format!("{} Error creating file.", "Error:".bright_red()));

    let mut install_file = File::create(&install_script_file)

        .expect(&format!("{} Error creating file.", "Error:".bright_red()));

    // Run for Homebrew
    let (uninstall_lines, install_lines) = generate_scripts::<Homebrew>();

    for line in uninstall_lines {
        uninstall_file.write_all(line.as_bytes()).expect(&format!(
            "{} Error writing to uninstall script.",
            "Error:".bright_red()
        ));

        info!(
        "{} Uninstall script generated at: {}",
        "Success:".bright_green(),
        uninstall_script_file
    );
    }

    for line in install_lines {
        install_file.write_all(line.as_bytes()).expect(&format!(
            "{} Error writing to install script.",
            "Error:".bright_red()
        ));
        info!(
        "{} Install script generated at: {}",
        "Success:".bright_green(),
        install_script_file
    );
    }
    // for t in 0..=5 {
    //     std::thread::spawn(|| {});
    // }
}
// ... (Cargo implementation remains the same)

pub fn install_command_cargo(package: &str) -> String {
    format!("cargo install {}\n", package)
}

fn run_command(command: &str, args: &[&str]) -> Vec<String> {
    let output = std::process::Command::new(command)
        .args(args)
        .output()
        .expect(&format!(
            "{} Error running {} command",
            "Error:".bright_red(),
            command
        ));

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect()
}

fn run_command_contains(command: &str, args: &[&str], pattern: &str) -> bool {
    let output = run_command(command, args);
    output.iter().any(|line| line.contains(pattern))
}

fn generate_scripts<T: PackageManager + std::marker::Sync>() -> (Vec<String>, Vec<String>) {
    let package_manager = T::new();
    let packages = package_manager.get_installed_packages();

    let results: Vec<_> = packages.par_iter().filter_map(|package| {
        if package_manager.has_rust_dependency(package) {
            if let Some(version) = package_manager.get_version(package) {
                let uninstall_line = package_manager.uninstall_command(package);
                let install_line = install_command_cargo(&format!("{}@{}", package, version));
                return Some((uninstall_line, install_line));
            }
        }
        None
    }).collect();

    let (uninstall_lines, install_lines): (Vec<_>, Vec<_>) = results.into_iter().unzip();

    println!(
        "{} Installing packages from Homebrew...",
        "Info:".bright_cyan()
    );

    (uninstall_lines, install_lines)
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
