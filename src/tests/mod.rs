use std::path::PathBuf;

mod wine;

#[cfg(feature = "wine-fonts")]
mod fonts;

#[cfg(feature = "wine-proton")]
mod proton;

#[cfg(feature = "dxvk")]
mod dxvk;

pub fn get_test_dir() -> PathBuf {
    std::env::temp_dir().join("wincompatlib-test")
}
