use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio, Child};

use crate::wine::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Winetricks {
    /// Path to the `winetricks` script
    pub winetricks: PathBuf,

    /// Path to the `wineserver` binary
    pub wineserver: Option<PathBuf>,

    /// Path to the `wine` binary
    pub wineloader: Option<PathBuf>,

    /// Path to the wine prefix
    pub wineprefix: Option<PathBuf>,

    /// Wine architecture
    pub arch: Option<WineArch>
}

impl Winetricks {
    #[inline]
    pub fn new(winetricks: impl Into<PathBuf>) -> Self {
        Self {
            winetricks: winetricks.into(),
            wineserver: None,
            wineloader: None,
            wineprefix: None,
            arch: None
        }
    }

    #[inline]
    pub fn from_wine(winetricks: impl Into<PathBuf>, wine: impl AsRef<Wine>) -> Self {
        Self {
            winetricks: winetricks.into(),
            wineserver: Some(wine.as_ref().wineserver()),
            wineloader: Some(wine.as_ref().wineloader().to_path_buf()),
            wineprefix: wine.as_ref().prefix.clone(),
            arch: wine.as_ref().arch
        }
    }

    #[inline]
    pub fn with_server(self, wineserver: impl Into<PathBuf>) -> Self {
        Self {
            wineserver: Some(wineserver.into()),
            ..self
        }
    }

    #[inline]
    pub fn with_loader(self, wineloader: impl Into<PathBuf>) -> Self {
        Self {
            wineloader: Some(wineloader.into()),
            ..self
        }
    }

    #[inline]
    pub fn with_prefix(self, wineprefix: impl Into<PathBuf>) -> Self {
        Self {
            wineprefix: Some(wineprefix.into()),
            ..self
        }
    }

    #[inline]
    pub fn with_arch(self, arch: WineArch) -> Self {
        Self {
            arch: Some(arch),
            ..self
        }
    }

    #[inline]
    pub fn install(&self, component: impl AsRef<str>) -> std::io::Result<Child> {
        self.install_args_with_env(component, ["-q"], [])
    }

    #[inline]
    pub fn install_args<T, S>(&self, component: impl AsRef<str>, args: T) -> std::io::Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.install_args_with_env(component, args, [])
    }

    pub fn install_args_with_env<T, K, S>(&self, component: impl AsRef<str>, args: T, envs: K) -> std::io::Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        let mut command = Command::new("bash");

        command
            .arg(&self.winetricks)
            .arg(component.as_ref());

        if let Some(server) = &self.wineserver {
            command.env("WINESERVER", server);
        }

        if let Some(loader) = &self.wineloader {
            command.env("WINELOADER", loader);
            command.env("WINE", loader);

            // Not really needed but I anyway will set it
            if self.arch == Some(WineArch::Win64) {
                command.env("WINE64", loader);
            }
        }

        if let Some(prefix) = &self.wineprefix {
            command.env("WINEPREFIX", prefix);
        }

        if let Some(arch) = self.arch {
            command.env("WINEARCH", arch.to_str());
        }

        command
            .args(args)
            .envs(envs)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}
