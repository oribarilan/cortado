use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static IS_DEV: OnceLock<bool> = OnceLock::new();

/// Initialize the app environment from the Tauri config identifier.
///
/// Must be called once at startup before any config loading.
/// Dev mode is detected by an identifier ending in `.dev`.
pub fn init(identifier: &str) {
    let is_dev = identifier.ends_with(".dev");
    IS_DEV.get_or_init(|| is_dev);

    let dir_name = if is_dev { "cortado-dev" } else { "cortado" };
    let home = dirs::home_dir().expect("could not resolve home directory");
    CONFIG_DIR.get_or_init(|| home.join(".config").join(dir_name));
}

/// Whether this is a dev build (identifier ends with `.dev`).
///
/// Falls back to `false` if [`init`] was not called (e.g. in unit tests).
pub fn is_dev() -> bool {
    *IS_DEV.get_or_init(|| false)
}

/// The config directory for this build variant.
///
/// Falls back to `~/.config/cortado` if [`init`] was not called (e.g. in unit tests).
pub fn config_dir() -> &'static Path {
    CONFIG_DIR.get_or_init(|| {
        dirs::home_dir()
            .expect("could not resolve home directory")
            .join(".config")
            .join("cortado")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_fallback_is_production() {
        // Without init(), config_dir() should return the production default.
        let dir = config_dir();
        assert!(
            dir.ends_with(".config/cortado"),
            "expected production fallback, got {:?}",
            dir
        );
    }

    #[test]
    fn is_dev_fallback_is_false() {
        assert!(!is_dev());
    }
}
