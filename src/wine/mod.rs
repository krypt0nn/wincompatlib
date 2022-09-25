use std::collections::HashMap;
use std::ffi::OsString;
use std::os::unix::prelude::OsStringExt;
use std::path::PathBuf;
use std::io::Result;
use std::process::{Command, Stdio, Child, Output};

mod with_ext;
mod boot_ext;

pub use with_ext::WineWithExt;
pub use boot_ext::WineBootExt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WineArch {
    Win32,
    Win64
}

impl WineArch {
    pub fn from_str(arch: &str) -> Option<Self> {
        match arch {
            "win32" => Some(Self::Win32),
            "win64" => Some(Self::Win64),
            _ => None
        }
    }

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
    pub fn new<T: Into<PathBuf>>(binary: T, prefix: Option<T>, arch: Option<WineArch>, wineboot: Option<T>, wineserver: Option<T>, wineloader: WineLoader) -> Self {
        Wine {
            binary: binary.into(),
            prefix: prefix.and_then(|value| Some(value.into())),
            arch,
            wineboot: wineboot.and_then(|value| Some(value.into())),
            wineserver: wineserver.and_then(|value| Some(value.into())),
            wineloader
        }
    }

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
    pub fn binary(&self) -> PathBuf {
        self.binary.clone()
    }

    fn get_inner_binary(&self, binary: &str) -> PathBuf {
        if let Some(parent) = self.binary.parent() {
            let binary_path = parent.join(binary);

            if binary_path.exists() {
                return binary_path;
            }
        }

        PathBuf::from(binary)
    }

    /// Get path to wineboot binary, or "wineboot" if not specified
    /// 
    /// If wine binary is specified (so not system), then function will try to find wineboot binary inside of this wine's folder
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// use std::path::PathBuf;
    /// 
    /// assert_eq!(Wine::default().wineboot(), PathBuf::from("wineboot"));
    /// assert_eq!(Wine::from_binary("/wine_build/wine").wineboot(), PathBuf::from("/wine_build/wineboot"));
    /// assert_eq!(Wine::from_binary("/wine_build_without_wineboot/wine").wineboot(), PathBuf::from("wineboot"));
    /// ```
    pub fn wineboot(&self) -> PathBuf {
        self.wineboot.clone().unwrap_or(self.get_inner_binary("wineboot"))
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
    /// assert_eq!(Wine::default().wineserver(), PathBuf::from("wineserver"));
    /// assert_eq!(Wine::from_binary("/wine_build/wine").wineserver(), PathBuf::from("/wine_build/wineserver"));
    /// assert_eq!(Wine::from_binary("/wine_build_without_wineserver/wine").wineserver(), PathBuf::from("wineserver"));
    /// ```
    pub fn wineserver(&self) -> PathBuf {
        self.wineserver.clone().unwrap_or(self.get_inner_binary("wineserver"))
    }

    /// Get path to wine binary, or "wine" if not specified (`WineLoader::Default`)
    pub fn wineloader(&self) -> PathBuf {
        match &self.wineloader {
            WineLoader::Default => PathBuf::from("wine"),
            WineLoader::Current => self.binary.clone(),
            WineLoader::Custom(path) => path.clone()
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

    /// Execute some binary using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run("/your/executable");
    /// ```
    pub fn run<T: Into<PathBuf>>(&self, binary: T) -> Result<Child> {
        let mut command = Command::new(&self.binary);

        command.arg(binary.into())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(prefix) = &self.prefix {
            command.env("WINEPREFIX", prefix);
        }

        if let Some(arch) = self.arch {
            command.env("WINEARCH", arch.to_str());
        }

        if let Some(server) = &self.wineserver {
            command.env("WINESERVER", server);
        }

        match &self.wineloader {
            WineLoader::Default => (),
            WineLoader::Current => {
                command.env("WINELOADER", &self.binary);
            },
            WineLoader::Custom(path) => {
                command.env("WINELOADER", path);
            }
        }

        command.spawn()
    }

    #[cfg(feature = "dxvk")]
    /// Run `Dxvk::install` with parameters from current Wine struct. Will try to use system-wide binaries if some not specified
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let output = Wine::from_binary("/path/to/wine")
    ///     .with_arch(WineArch::Win64)
    ///     .install_dxvk("/path/to/setup_dxvk.sh", "/path/to/prefix")
    ///     .expect("Failed to install DXVK");
    /// 
    /// println!("Installing output: {}", String::from_utf8_lossy(&output.stdout));
    /// ```
    pub fn install_dxvk<T: Into<PathBuf>>(&self, setup_script: T, prefix: T) -> Result<Output> {
        super::dxvk::Dxvk::install(
            setup_script.into(),
            prefix.into(),
            self.binary.clone(),
            self.get_inner_binary("wine64"),
            self.wineboot.clone().unwrap_or(
                self.get_inner_binary("wineboot")
            ),
            self.wineserver.clone().unwrap_or(
                self.get_inner_binary("wineserver")
            )
        )
    }

    #[cfg(feature = "dxvk")]
    /// Run `Dxvk::uninstall` with parameters from current Wine struct. Will try to use system-wide binaries if some not specified
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let output = Wine::from_binary("/path/to/wine")
    ///     .with_arch(WineArch::Win64)
    ///     .uninstall_dxvk("/path/to/setup_dxvk.sh", "/path/to/prefix")
    ///     .expect("Failed to uninstall DXVK");
    /// 
    /// println!("Uninstalling output: {}", String::from_utf8_lossy(&output.stdout));
    /// ```
    pub fn uninstall_dxvk<T: Into<PathBuf>>(&self, setup_script: T, prefix: T) -> Result<Output> {
        super::dxvk::Dxvk::uninstall(
            setup_script.into(),
            prefix.into(),
            self.binary.clone(),
            self.get_inner_binary("wine64"),
            self.wineboot.clone().unwrap_or(
                self.get_inner_binary("wineboot")
            ),
            self.wineserver.clone().unwrap_or(
                self.get_inner_binary("wineserver")
            )
        )
    }
}
