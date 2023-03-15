use std::path::Path;

use super::*;

#[cfg(feature = "wine-proton")]
pub mod proton;

pub trait Bundle {
    /// Get absolute path to the wine bundle
    fn path(&self) -> &Path;

    /// Get `Wine` struct from current bundle
    fn wine(&self) -> &Wine;
}
