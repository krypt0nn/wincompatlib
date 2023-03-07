use std::path::PathBuf;

use super::*;

pub trait WineWithExt {
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self;
    fn with_arch(self, arch: WineArch) -> Self;
    fn with_boot<T: Into<PathBuf>>(self, boot: T) -> Self;
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self;
    fn with_loader(self, loader: WineLoader) -> Self;
}

impl WineWithExt for Wine {
    /// Add path to wine prefix
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_prefix("/path/to/prefix");
    /// ```
    #[inline]
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        Self {
            prefix: Some(prefix.into()),
            ..self
        }
    }

    /// Add wine architecture
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_arch(WineArch::Win64);
    /// ```
    #[inline]
    fn with_arch(self, arch: WineArch) -> Self {
        Self {
            arch: Some(arch),
            ..self
        }
    }

    /// Add wineboot binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_boot("wineboot");
    /// ```
    #[inline]
    fn with_boot<T: Into<PathBuf>>(self, boot: T) -> Self {
        Self {
            wineboot: Some(boot.into()),
            ..self
        }
    }

    /// Add wineserver binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_server("wineserver");
    /// ```
    #[inline]
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self {
        Self {
            wineserver: Some(server.into()),
            ..self
        }
    }

    /// Add wineloader binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_loader(WineLoader::Custom(std::path::PathBuf::from("wine")));
    /// ```
    #[inline]
    fn with_loader(self, loader: WineLoader) -> Self {
        Self {
            wineloader: loader,
            ..self
        }
    }
}
