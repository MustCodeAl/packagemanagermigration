use crate::packmanager::PackageManager;

struct Cargo;


impl PackageManager for Cargo {
    fn new() -> Self {
        Self
    }

    fn get_installed_packages(&self) -> Vec<String> {
        Vec::new()
    }

    fn has_rust_dependency(&self, package: &str) -> bool {
        true
    }

    fn get_version(&self, package: &str) -> Option<String> {
        let output = crate::run_command("cargo", &["search", "--limit", "1", "--offline", package]);

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
