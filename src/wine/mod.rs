use std::collections::HashMap;
use std::ffi::{OsString, OsStr};
use std::os::unix::prelude::OsStringExt;
use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind, Result};
use std::process::{Command, Stdio, Output};

mod with_ext;
mod boot_ext;
mod run_ext;

pub use with_ext::WineWithExt;
pub use boot_ext::WineBootExt;
pub use run_ext::WineRunExt;

pub use derive_builder::Builder;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WineArch {
    Win32,
    Win64
}

impl WineArch {
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn from_str(arch: &str) -> Option<Self> {
        match arch {
            "win32" | "x32" | "32" => Some(Self::Win32),
            "win64" | "x64" | "64" => Some(Self::Win64),
            _ => None
        }
    }

    #[inline]
    pub fn to_str(&self) -> &str {
        match self {
            Self::Win32 => "win32",
            Self::Win64 => "win64"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WineLoader {
    /// Set `WINELOADER` variable as binary specified in `Wine` struct
    Current,

    /// Don't set `WINELOADER` variable, so wine will try to use system-wide binary
    Default,

    /// Set custom `WINELOADER` variable
    Custom(PathBuf)
}

impl Default for WineLoader {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wine {
    binary: PathBuf,

    /// Specifies `WINEPREFIX` variable
    pub prefix: Option<PathBuf>,

    /// Specifies `WINEARCH` variable
    pub arch: Option<WineArch>,

    /// Path to wineboot binary
    pub wineboot: Option<PathBuf>,

    /// Specifies `WINESERVER` variable
    pub wineserver: Option<PathBuf>,

    /// Specifies `WINELOADER` variable
    pub wineloader: WineLoader
}

impl Default for Wine {
    fn default() -> Self {
        Self::from_binary("wine")
    }
}

impl Wine {
    #[inline]
    pub fn new<T: Into<PathBuf>>(
        binary: T,
        prefix: Option<T>,
        arch: Option<WineArch>,
        wineboot: Option<T>,
        wineserver: Option<T>,
        wineloader: WineLoader
    ) -> Self {
        Wine {
            binary: binary.into(),
            prefix: prefix.map(|value| value.into()),
            arch,
            wineboot: wineboot.map(|value| value.into()),
            wineserver: wineserver.map(|value| value.into()),
            wineloader
        }
    }

    #[inline]
    pub fn from_binary<T: Into<PathBuf>>(binary: T) -> Self {
        Self::new(binary, None, None, None, None, WineLoader::default())
    }

    /// Try to get version of provided wine binary. Runs command: `wine --version`
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// match Wine::default().version() {
    ///     Ok(version) => println!("Wine version: {:?}", version),
    ///     Err(err) => eprintln!("Wine is not available: {}", err)
    /// }
    /// ```
    pub fn version(&self) -> Result<OsString> {
        let output = Command::new(&self.binary)
           .arg("--version")
           .stdout(Stdio::piped())
           .stderr(Stdio::null())
           .output()?;

        Ok(OsString::from_vec(output.stdout))
    }

    /// Get wine binary path
    #[inline]
    pub fn binary(&self) -> &Path {
        self.binary.as_path()
    }

    fn get_inner_binary(&self, binary: &str) -> Option<PathBuf> {
        if let Some(parent) = self.binary.parent() {
            let binary_path = parent.join(binary);

            if binary_path.exists() {
                return Some(binary_path);
            }
        }

        None
    }

    /// Get path to wineboot binary, or current wine binary path if not specified
    /// 
    /// If wine binary is specified (so not system), then function will try to find wineboot binary inside of this wine's folder
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// use std::path::PathBuf;
    /// 
    /// assert_eq!(Wine::from_binary("wine_build_with_wineboot/wine").wineboot(), PathBuf::from("wine_build_with_wineboot/wineboot"));
    /// assert_eq!(Wine::from_binary("wine_build_without_wineboot/wine").wineboot(), PathBuf::from("wine_build_without_wineboot/wine"));
    /// ```
    #[inline]
    pub fn wineboot(&self) -> PathBuf {
        self.wineboot.clone()
            .unwrap_or_else(|| self.get_inner_binary("wineboot")
            .unwrap_or_else(|| self.binary.clone()))
    }

    /// Get path to wineserver binary, or "wineserver" if not specified
    /// 
    /// If wine binary is specified (so not system), then function will try to find wineserver binary inside of this wine's folder
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// use std::path::PathBuf;
    /// 
    /// assert_eq!(Wine::from_binary("wine_build_with_wineserver/wine").wineserver(), PathBuf::from("wine_build_with_wineserver/wineserver"));
    /// assert_eq!(Wine::from_binary("wine_build_without_wineserver/wine").wineserver(), PathBuf::from("wineserver"));
    /// ```
    #[inline]
    pub fn wineserver(&self) -> PathBuf {
        self.wineserver.clone()
            .unwrap_or_else(|| self.get_inner_binary("wineserver")
            .unwrap_or_else(|| PathBuf::from("wineserver")))
    }

    /// Get path to wine binary, or "wine" if not specified (`WineLoader::Default`)
    #[inline]
    pub fn wineloader(&self) -> &Path {
        match &self.wineloader {
            WineLoader::Default => Path::new("wine"),
            WineLoader::Current => self.binary.as_path(),
            WineLoader::Custom(path) => path
        }
    }

    /// Get environment variables map from current struct's values
    /// 
    /// ```
    /// use wincompatlib::prelude::*;
    /// 
    /// use std::process::Command;
    /// 
    /// let wine = Wine::default().with_arch(WineArch::Win64);
    /// 
    /// Command::new(wine.binary())
    ///     .envs(wine.get_envs())
    ///     .spawn();
    /// ```
    pub fn get_envs(&self) -> HashMap<&str, OsString> {
        let mut env = HashMap::new();

        if let Some(prefix) = &self.prefix {
            env.insert("WINEPREFIX", prefix.as_os_str().to_os_string());
        }

        if let Some(arch) = self.arch {
            env.insert("WINEARCH", match arch {
                WineArch::Win32 => OsString::from("win32"),
                WineArch::Win64 => OsString::from("win64")
            });
        }

        if let Some(server) = &self.wineserver {
            env.insert("WINESERVER", server.as_os_str().to_os_string());
        }

        match &self.wineloader {
            WineLoader::Default => (),
            WineLoader::Current => {
                env.insert("WINELOADER", self.binary.as_os_str().to_os_string());
            },
            WineLoader::Custom(path) => {
                env.insert("WINELOADER", path.as_os_str().to_os_string());
            }
        }

        env
    }

    #[cfg(feature = "dxvk")]
    /// Run `Dxvk::install` with parameters from current Wine struct. Will try to use system-wide binaries if some not specified
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::from_binary("/path/to/wine")
    ///     .with_arch(WineArch::Win64)
    ///     .install_dxvk("/path/to/dxvk-2.1", InstallParams::default())
    ///     .expect("Failed to install DXVK 2.1");
    /// ```
    #[inline]
    pub fn install_dxvk<T: Into<PathBuf>>(&self, dxvk_folder: T, params: super::dxvk::InstallParams) -> Result<()> {
        super::dxvk::Dxvk::install(self, dxvk_folder, params)
    }

    #[cfg(feature = "dxvk")]
    /// Run `Dxvk::uninstall` with parameters from current Wine struct. Will try to use system-wide binaries if some not specified
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::from_binary("/path/to/wine")
    ///     .with_arch(WineArch::Win64)
    ///     .uninstall_dxvk(InstallParams::default())
    ///     .expect("Failed to uninstall DXVK");
    /// ```
    #[inline]
    pub fn uninstall_dxvk(&self, params: super::dxvk::InstallParams) -> Result<()> {
        super::dxvk::Dxvk::uninstall(self, params)
    }
}
