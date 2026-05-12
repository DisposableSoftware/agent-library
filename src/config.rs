use std::path::PathBuf;

/// Resolve the library storage path.
///
/// Priority:
/// 1. `AGENT_LIBRARY_PATH` env var (all platforms)
/// 2. `XDG_CONFIG_HOME` env var (Linux only)
/// 3. `~/.config/agent-library` (macOS + Linux fallback)
pub fn resolve_library_path() -> PathBuf {
    if let Ok(p) = std::env::var("AGENT_LIBRARY_PATH") {
        return PathBuf::from(p);
    }

    #[cfg(target_os = "linux")]
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("agent-library");
    }

    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".config").join("agent-library")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: run a closure with a specific env var set, then restore.
    // Tests manipulating env vars must not run in parallel with each other.
    fn with_env<F: FnOnce()>(key: &str, val: &str, f: F) {
        let old = std::env::var(key).ok();
        // SAFETY: single-threaded test context
        unsafe { std::env::set_var(key, val) };
        f();
        match old {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
    }

    fn without_env<F: FnOnce()>(key: &str, f: F) {
        let old = std::env::var(key).ok();
        // SAFETY: single-threaded test context
        unsafe { std::env::remove_var(key) };
        f();
        if let Some(v) = old {
            unsafe { std::env::set_var(key, v) };
        }
    }

    #[test]
    fn agent_library_path_env_var_wins() {
        with_env("AGENT_LIBRARY_PATH", "/tmp/override", || {
            assert_eq!(resolve_library_path(), PathBuf::from("/tmp/override"));
        });
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_xdg_config_home() {
        without_env("AGENT_LIBRARY_PATH", || {
            with_env("XDG_CONFIG_HOME", "/tmp/xdg", || {
                assert_eq!(
                    resolve_library_path(),
                    PathBuf::from("/tmp/xdg/agent-library")
                );
            });
        });
    }

    #[test]
    fn fallback_to_home_config() {
        without_env("AGENT_LIBRARY_PATH", || {
            without_env("XDG_CONFIG_HOME", || {
                let result = resolve_library_path();
                let home = std::env::var("HOME").unwrap();
                assert_eq!(result, PathBuf::from(&home).join(".config").join("agent-library"));
            });
        });
    }
}
