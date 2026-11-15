//! Target specifications for provided images whose triple is not built into rustc.
//!
//! A non-builtin triple needs both `RUST_TARGET_PATH` and `-Zunstable-options`, and
//! rustc requires them for every invocation, including the target info probes cargo
//! performs before the build starts. Supplying both here means users do not have to.

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::Result;

/// The flag rustc requires before it will load a target specification.
pub const CUSTOM_TARGET_FLAG: &str = "-Zunstable-options";

const RISCV32IMAC_UNKNOWN_LINUX_MUSL: &str =
    include_str!("target_specs/riscv32imac-unknown-linux-musl.json");

/// Returns the target specification cross ships for `triple`, if it is not built in.
pub fn spec(triple: &str) -> Option<&'static str> {
    match triple {
        "riscv32imac-unknown-linux-musl" => Some(RISCV32IMAC_UNKNOWN_LINUX_MUSL),
        _ => None,
    }
}

/// Writes the specification for `triple` to a temporary directory, which can then be
/// passed to rustc as `RUST_TARGET_PATH`.
pub fn spec_path(triple: &str) -> Result<Option<PathBuf>> {
    let Some(spec) = spec(triple) else {
        return Ok(None);
    };
    let dir = env::temp_dir().join("cross-target-specs").join(triple);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(format!("{triple}.json")), spec)?;
    Ok(Some(dir))
}

/// Returns the `RUSTFLAGS` value needed to build for a custom target.
pub fn rustflags(current: Option<&str>) -> Option<String> {
    let current = current.unwrap_or_default();
    if current.split_whitespace().any(|f| f == CUSTOM_TARGET_FLAG) {
        None
    } else if current.is_empty() {
        Some(CUSTOM_TARGET_FLAG.to_owned())
    } else {
        Some(format!("{current} {CUSTOM_TARGET_FLAG}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_is_known() {
        assert!(spec("riscv32imac-unknown-linux-musl").is_some());
        assert!(spec("x86_64-unknown-linux-gnu").is_none());
    }

    #[test]
    fn rustflags_appends_once() {
        assert_eq!(rustflags(None), Some("-Zunstable-options".to_owned()));
        assert_eq!(
            rustflags(Some("-C opt-level=1")),
            Some("-C opt-level=1 -Zunstable-options".to_owned())
        );
        assert_eq!(rustflags(Some("-Zunstable-options")), None);
    }

    /// the copy shipped in the image must not drift from the one embedded here
    #[test]
    fn matches_docker_copy() {
        assert_eq!(
            RISCV32IMAC_UNKNOWN_LINUX_MUSL,
            include_str!("../docker/target-specs/riscv32imac-unknown-linux-musl.json")
        );
    }
}
