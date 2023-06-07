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
    pub wineprefix: PathBuf,

    /// Wine architecture
    pub arch: WineArch
}

impl Winetricks {
    #[inline]
    pub fn new(winetricks: impl Into<PathBuf>) -> Self {
        Self::from_wine(winetricks, Wine::default())
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
            wineprefix: wineprefix.into(),
            ..self
        }
    }

    #[inline]
    pub fn with_arch(self, arch: impl Into<WineArch>) -> Self {
        Self {
            arch: arch.into(),
            ..self
        }
    }

    #[inline]
    pub fn install(&self, component: impl AsRef<str>) -> anyhow::Result<Child> {
        self.install_args_with_env(component, ["-q"], [])
    }

    #[inline]
    pub fn install_args<T, S>(&self, component: impl AsRef<str>, args: T) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.install_args_with_env(component, args, [])
    }

    pub fn install_args_with_env<T, K, S>(&self, component: impl AsRef<str>, args: T, envs: K) -> anyhow::Result<Child>
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
            if self.arch == WineArch::Win64 {
                command.env("WINE64", loader);
            }
        }

        command.env("WINEPREFIX", &self.wineprefix);
        command.env("WINEARCH", self.arch.to_str());

        Ok(command
            .args(args)
            .envs(envs)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?)
    }
}
