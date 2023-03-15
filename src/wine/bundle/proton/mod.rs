use std::path::{Path, PathBuf};
use std::process::Child;

use crate::wine::*;
use super::Bundle;

mod run_in_prefix_ext;
mod wait_for_exit_and_run_ext;

pub use run_in_prefix_ext::RunInPrefixExt;
pub use wait_for_exit_and_run_ext::WaitForExitAndRunExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proton {
    path: PathBuf,
    wine: Wine,

    /// Path to proton prefix. Sets `STEAM_COMPAT_DATA_PATH` environment variable
    /// 
    /// Approximate folder structure:
    /// 
    /// ```txt
    /// .
    /// ├── pfx.lock
    /// ├── tracked_files
    /// ├── version
    /// └── pfx/
    ///     └── wine prefix files
    /// ```
    pub proton_prefix: Option<PathBuf>,

    /// Path to folder with steam client folders
    /// 
    /// Sets `STEAM_COMPAT_CLIENT_INSTALL_PATH` environment variable
    pub steam_client_path: Option<PathBuf>,

    /// Sets `SteamAppId` environment variable
    pub steam_app_id: u32,

    /// Path to python interpreter. `python3` by default
    pub python: PathBuf
}

impl Bundle for Proton {
    #[inline]
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    #[inline]
    fn wine(&self) -> &Wine {
        &self.wine
    }
}

impl Proton {
    pub fn new<T: Into<PathBuf>>(path: T, proton_prefix: Option<T>) -> Self {
        let path = path.into();

        let (proton_prefix, wine_prefix) = match proton_prefix {
            Some(proton_prefix) => {
                let proton_prefix = proton_prefix.into();
                let wine_prefix = proton_prefix.join("pfx");

                (Some(proton_prefix), Some(wine_prefix))
            }

            None => (None, None)
        };

        Self {
            wine: Wine::new(
                path.join("files/bin/wine64"),
                wine_prefix,
                Some(WineArch::Win64),
                None,
                Some(path.join("files/bin/wineserver")),
                WineLoader::Current
            ),
            path,
            proton_prefix,
            steam_client_path: None,
            steam_app_id: 0,
            python: PathBuf::from("python3")
        }
    }

    /// Get environment variables map from current struct's values
    /// 
    /// Includes inner wine variables
    /// 
    /// Can contain (if specified in current struct):
    /// 
    /// - Wine variables (`self.wine().get_envs()`)
    /// - `STEAM_COMPAT_DATA_PATH`
    /// - `STEAM_COMPAT_CLIENT_INSTALL_PATH`
    /// - `SteamAppId` (always, 0 by default)
    pub fn get_envs(&self) -> HashMap<&str, OsString> {
        let mut env = self.wine.get_envs();

        if let Some(compat_data) = &self.proton_prefix {
            env.insert("STEAM_COMPAT_DATA_PATH", compat_data.into());
        }

        if let Some(steam_client) = &self.steam_client_path {
            env.insert("STEAM_COMPAT_CLIENT_INSTALL_PATH", steam_client.into());
        }

        env.insert("SteamAppId", self.steam_app_id.to_string().into());

        env
    }
}

impl WineWithExt for Proton {
    /// Add path to proton directory
    /// 
    /// This method will try to automatically update both `self.proton_prefix` and `wine.prefix`
    /// 
    /// By default `prefix` should point to proton prefix, so wine prefix will be in `prefix/pfx`
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        let prefix = prefix.into();

        // `prefix` is proton prefix, `prefix/pfx` is wine prefix (default)
        if !prefix.exists() || prefix.join("pfx").exists() || !prefix.join("drive_c").exists() {
            Self {
                wine: self.wine.with_prefix(prefix.join("pfx")),
                proton_prefix: Some(prefix),
                ..self
            }
        }

        // `prefix` is wine prefix, `prefix/../` is proton prefix (reversed order)
        else {
            Self {
                proton_prefix: prefix.parent().map(|path| path.to_path_buf()),
                wine: self.wine.with_prefix(prefix),
                ..self
            }
        }
    }

    /// Add wine architecture
    #[inline]
    fn with_arch(self, arch: WineArch) -> Self {
        Self {
            wine: self.wine.with_arch(arch),
            ..self
        }
    }

    /// Add wineboot binary
    #[inline]
    fn with_boot(self, boot: WineBoot) -> Self {
        Self {
            wine: self.wine.with_boot(boot),
            ..self
        }
    }

    /// Add wineserver binary
    #[inline]
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self {
        Self {
            wine: self.wine.with_server(server),
            ..self
        }
    }

    /// Add wineloader binary
    #[inline]
    fn with_loader(self, loader: WineLoader) -> Self {
        Self {
            wine: self.wine.with_loader(loader),
            ..self
        }
    }
}

impl WineBootExt for Proton {
    /// Get base `wineboot` command. Will return `wine wineboot` if `self.wineboot()` is `None`
    #[inline]
    fn wineboot_command(&self) -> Command {
        self.wine.wineboot_command()
    }

    /// Create (or update existing) wine prefix. Runs `wineboot -u` command
    #[inline]
    fn update_prefix<T: Into<PathBuf>>(&self, path: T) -> Result<Output> {
        self.wine.update_prefix(path)
    }

    /// Stop running processes. Runs `wineboot -k` command, or `wineboot -f` if `force = true`
    #[inline]
    fn stop_processes(&self, force: bool) -> Result<Output> {
        self.wine.stop_processes(force)
    }

    /// Imitate windows restart. Runs `wineboot -r` command
    #[inline]
    fn restart(&self) -> Result<Output> {
        self.wine.restart()
    }

    /// Imitate windows shutdown. Runs `wineboot -s` command
    #[inline]
    fn shutdown(&self) -> Result<Output> {
        self.wine.shutdown()
    }

    /// End wineboot session. Runs `wineboot -e` command
    #[inline]
    fn end_session(&self) -> Result<Output> {
        self.wine.end_session()
    }
}

impl WineRunExt for Proton {
    /// Run the game using proton
    #[inline]
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child> {
        self.run_args_with_env([binary], [])
    }

    /// Run the game using proton
    /// 
    /// Note that it doesn't accept several arguments. You should use `[binary]` here only.
    /// This syntax remains here only because of `WineRunExt` trait
    #[inline]
    fn run_args<T, S>(&self, args: T) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.run_args_with_env(args, [])
    }

    /// Run the game using proton
    /// 
    /// Note that it doesn't accept several arguments. You should use `[binary]` here only.
    /// This syntax remains here only because of `WineRunExt` trait
    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        Command::new(self.python.as_os_str())
            .arg(self.path.join("proton"))
            .arg("run")
            .args(args)
            .envs(self.get_envs())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(envs)
            .spawn()
    }

    #[inline]
    fn winepath(&self, path: &str) -> Result<PathBuf> {
        self.wine.winepath(path)
    }
}
