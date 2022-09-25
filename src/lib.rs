pub mod wine;

#[cfg(feature = "dxvk")]
pub mod dxvk;

#[cfg(test)]
mod test;

pub mod prelude {
    pub use super::wine::*;

    #[cfg(feature = "dxvk")]
    pub use super::dxvk::*;
}
