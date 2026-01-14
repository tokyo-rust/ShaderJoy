//! ShaderJoy Core Library
//!
//! Platform-agnostic core logic for shader generation, evolution, and management.

use crate::config::{config_search_paths, project_dirs};

pub mod audio;
pub mod config;
pub mod error;
pub mod generation;
pub mod shaders;
pub mod storage;

/// Returns a human-readable description of configuration search paths.
///
/// If no config file exists in any search path, this also logs the info at `info` level.
pub fn describe_config_paths() -> String {
    use std::fmt::Write;

    let paths = config_search_paths();
    let mut output = String::from(
        "Configuration search paths (current directory takes priority, then platform config dir):\n",
    );
    let mut any_exists = false;

    for path in &paths {
        let exists = path.exists();
        if exists {
            any_exists = true;
        }
        let status = if exists { "exists" } else { "not found" };
        let _ = writeln!(output, "  - {} ({})", path.display(), status);
    }

    if let Some(dirs) = project_dirs() {
        let _ = writeln!(output, "Platform directories:");
        let _ = writeln!(output, "  config: {}", dirs.config_dir().display());
        let _ = writeln!(output, "  data:   {}", dirs.data_dir().display());
        let _ = writeln!(output, "  cache:  {}", dirs.cache_dir().display());
    } else {
        let _ = writeln!(output, "Platform directories: not available");
    }

    if !any_exists {
        tracing::info!("{}", output.trim_end());
    }

    output
}
