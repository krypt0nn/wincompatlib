pub mod wine;

#[cfg(feature = "dxvk")]
pub mod dxvk;

#[cfg(feature = "winetricks")]
pub mod winetricks;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use super::wine::*;

    #[cfg(feature = "wine-bundles")]
    pub use super::wine::bundle::Bundle as WineBundle;

    #[cfg(feature = "wine-proton")]
    pub use super::wine::bundle::proton::*;

    #[cfg(feature = "dxvk")]
    pub use super::dxvk::*;

    #[cfg(feature = "winetricks")]
    pub use super::winetricks::*;
}
