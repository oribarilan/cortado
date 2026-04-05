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
    CONFIG_DIR.get_or_init(|| resolve_config_dir(dir_name));
}

/// Whether this is a dev build (identifier ends with `.dev`).
///
/// Falls back to `false` if [`init`] was not called (e.g. in unit tests).
pub fn is_dev() -> bool {
    *IS_DEV.get_or_init(|| false)
}

/// The config directory for this build variant.
///
/// Resolution order (macOS):
/// 1. `$XDG_CONFIG_HOME/<dir_name>` if the env var is set and non-empty.
/// 2. `~/.config/<dir_name>` otherwise.
///
/// Note: we deliberately use XDG conventions on macOS, not Apple's
/// `~/Library/Application Support/`. The `dirs` crate's `config_dir()`
/// returns the Apple path, which is why we resolve manually.
///
/// Falls back to `~/.config/cortado` if [`init`] was not called (e.g. in unit tests).
pub fn config_dir() -> &'static Path {
    CONFIG_DIR.get_or_init(|| resolve_config_dir("cortado"))
}

/// Resolves the config directory for a given app directory name.
///
/// Checks `$XDG_CONFIG_HOME` first, falling back to `~/.config/`.
fn resolve_config_dir(dir_name: &str) -> PathBuf {
    resolve_config_dir_from(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), dir_name)
}

/// Pure config directory resolution — takes XDG value as parameter for testability.
fn resolve_config_dir_from(xdg_config_home: Option<&str>, dir_name: &str) -> PathBuf {
    if let Some(xdg) = xdg_config_home {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join(dir_name);
        }
    }
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".config")
        .join(dir_name)
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

    #[test]
    fn resolve_with_xdg_override() {
        let dir = resolve_config_dir_from(Some("/tmp/xdg-test"), "cortado");
        assert_eq!(dir, PathBuf::from("/tmp/xdg-test/cortado"));
    }

    #[test]
    fn resolve_with_xdg_override_dev() {
        let dir = resolve_config_dir_from(Some("/custom/config"), "cortado-dev");
        assert_eq!(dir, PathBuf::from("/custom/config/cortado-dev"));
    }

    #[test]
    fn resolve_without_xdg_falls_back_to_dot_config() {
        let dir = resolve_config_dir_from(None, "cortado");
        assert!(
            dir.ends_with(".config/cortado"),
            "expected ~/.config/cortado fallback, got {:?}",
            dir
        );
    }

    #[test]
    fn resolve_with_empty_xdg_falls_back() {
        let dir = resolve_config_dir_from(Some(""), "cortado");
        assert!(
            dir.ends_with(".config/cortado"),
            "empty XDG_CONFIG_HOME should fall back, got {:?}",
            dir
        );
    }
}
