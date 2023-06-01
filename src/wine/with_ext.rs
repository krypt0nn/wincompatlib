use std::path::PathBuf;

use super::*;

pub trait WineWithExt {
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self;
    fn with_arch(self, arch: WineArch) -> Self;
    fn with_boot(self, boot: WineBoot) -> Self;
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self;
    fn with_loader(self, loader: WineLoader) -> Self;
    fn with_wine_libs(self, wine_libs: WineSharedLibs) -> Self;
    fn with_gstreamer_libs(self, gstreamer_libs: GstreamerSharedLibs) -> Self;
}

impl WineWithExt for Wine {
    #[inline]
    /// Add path to wine prefix
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_prefix("/path/to/prefix");
    /// ```
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        Self {
            prefix: Some(prefix.into()),
            ..self
        }
    }

    #[inline]
    /// Add wine architecture
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_arch(WineArch::Win64);
    /// ```
    fn with_arch(self, arch: WineArch) -> Self {
        Self {
            arch: Some(arch),
            ..self
        }
    }

    #[inline]
    /// Add wineboot binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_boot(WineBoot::Unix(std::path::PathBuf::from("path/to/wineboot")));
    /// ```
    fn with_boot(self, boot: WineBoot) -> Self {
        Self {
            wineboot: Some(boot),
            ..self
        }
    }

    #[inline]
    /// Add wineserver binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_server("wineserver");
    /// ```
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self {
        Self {
            wineserver: Some(server.into()),
            ..self
        }
    }

    #[inline]
    /// Add wineloader binary
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// let wine = Wine::from_binary("wine")
    ///     .with_loader(WineLoader::Custom(std::path::PathBuf::from("wine")));
    /// ```
    fn with_loader(self, loader: WineLoader) -> Self {
        Self {
            wineloader: loader,
            ..self
        }
    }

    #[inline]
    /// Set wine shared libraries paths
    fn with_wine_libs(self, wine_libs: shared_libraries::Wine) -> Self {
        Self {
            wine_libs,
            ..self
        }
    }

    #[inline]
    /// Set gstreamer shared libraries paths
    fn with_gstreamer_libs(self, gstreamer_libs: shared_libraries::Gstreamer) -> Self {
        Self {
            gstreamer_libs,
            ..self
        }
    }
}
