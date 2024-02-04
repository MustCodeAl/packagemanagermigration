use crate::packmanager::PackageManager;

struct Apt;


impl PackageManager for Apt {
    fn new() -> Self {
        Self
    }
    fn get_installed_packages(&self) -> Vec<String> {
        crate::run_command("apt", &["list", "--installed"])
    }
    fn has_rust_dependency(&self, package: &str) -> bool {
        crate::run_command_contains("apt", &["show", package], "rust")
    }
    fn get_version(&self, package: &str) -> Option<String> {
        Option::from(format!(""))
    }
    fn uninstall_command(&self, package: &str) -> String {
        format!("apt remove {package}\n")
    }
    fn install_command(&self, package: &str) -> String {
        format!("cargo install {package}\n")
    }
}
