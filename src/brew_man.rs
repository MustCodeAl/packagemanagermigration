use crate::packmanager::PackageManager;

pub struct Homebrew;

impl PackageManager for Homebrew {
    fn new() -> Self {
        Self
    }

    fn get_installed_packages(&self) -> Vec<String> {
        crate::run_command("brew", &["list", "--formula"])
    }

    fn has_rust_dependency(&self, package: &str) -> bool {
        crate::run_command_contains("brew", &["info", package], "rust")
    }

    fn get_version(&self, package: &str) -> Option<String> {
        let output = crate::run_command("brew", &["info", "--json=v1", package]);
        let info: serde_json::Value =
            serde_json::from_str(&output.join("\n")).expect("Error parsing JSON");

        let version = info[0]["versions"]["stable"].as_str().map(String::from);

        version
    }

    fn uninstall_command(&self, package: &str) -> String {
        format!("brew uninstall {}\n", package)
    }

    fn install_command(&self, package: &str) -> String {
        format!("cargo install {}\n", package)
    }
}
