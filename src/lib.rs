pub mod wine;

#[cfg(feature = "dxvk")]
pub mod dxvk;

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
}
