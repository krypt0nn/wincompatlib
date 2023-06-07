use std::path::PathBuf;

use crate::wine::*;

pub trait WineWithExt {
    /// Add path to wine prefix
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_prefix("/path/to/prefix");
    /// ```
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self;

    /// Add wine architecture
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_arch(WineArch::Win64);
    /// ```
    fn with_arch(self, arch: WineArch) -> Self;

    /// Add wineboot binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_boot(WineBoot::Unix(std::path::PathBuf::from("path/to/wineboot")));
    /// ```
    fn with_boot(self, boot: WineBoot) -> Self;

    /// Add wineserver binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_server("wineserver");
    /// ```
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self;

    /// Add wineloader binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_loader(WineLoader::Custom(std::path::PathBuf::from("wine")));
    /// ```
    fn with_loader(self, loader: WineLoader) -> Self;

    /// Set wine shared libraries paths
    fn with_wine_libs(self, wine_libs: WineSharedLibs) -> Self;

    /// Set gstreamer shared libraries paths
    fn with_gstreamer_libs(self, gstreamer_libs: GstreamerSharedLibs) -> Self;
}

impl WineWithExt for Wine {
    #[inline]
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        Self {
            prefix: prefix.into(),
            ..self
        }
    }

    #[inline]
    fn with_arch(self, arch: WineArch) -> Self {
        Self {
            arch,
            ..self
        }
    }

    #[inline]
    fn with_boot(self, boot: WineBoot) -> Self {
        Self {
            wineboot: Some(boot),
            ..self
        }
    }

    #[inline]
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self {
        Self {
            wineserver: Some(server.into()),
            ..self
        }
    }

    #[inline]
    fn with_loader(self, loader: WineLoader) -> Self {
        Self {
            wineloader: loader,
            ..self
        }
    }

    #[inline]
    fn with_wine_libs(self, wine_libs: shared_libraries::Wine) -> Self {
        Self {
            wine_libs,
            ..self
        }
    }

    #[inline]
    fn with_gstreamer_libs(self, gstreamer_libs: shared_libraries::Gstreamer) -> Self {
        Self {
            gstreamer_libs,
            ..self
        }
    }
}
