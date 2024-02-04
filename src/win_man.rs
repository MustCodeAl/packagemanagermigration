use crate::packmanager::PackageManager;

struct Winget;


impl PackageManager for Winget {
    fn new() -> Self {
        Self
    }
     fn get_installed_packages(&self) -> Vec<String> {
        // --source "winget"
        crate::run_command("winget", &["list", "--source \"winget\""])
    }
     fn has_rust_dependency(&self, package: &str) -> bool {
        todo!("fix for winget")
    }
     fn get_version(&self, package: &str) -> Option<String> {
        Option::from(format!(""))
    }
    fn uninstall_command(&self, package: &str) -> String {
        format!("winget uninstall {package}")
    }
    fn install_command(&self, package: &str) -> String {
        format!("cargo install {package}")
    }
}
