use crate::brew_man::Homebrew;

pub trait PackageManager {
    fn new() -> Self;
    fn get_installed_packages(&self) -> Vec<String>;
    fn has_rust_dependency(&self, package: &str) -> bool;
    fn get_version(&self, package: &str) -> Option<String>;
    fn uninstall_command(&self, package: &str) -> String;
    fn install_command(&self, package: &str) -> String;
}

