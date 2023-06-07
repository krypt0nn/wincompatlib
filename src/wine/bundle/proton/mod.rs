use std::path::{Path, PathBuf};
use std::process::{Child, Output};
use std::ffi::OsStr;

use crate::wine::*;
use crate::wine::ext::*;
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

        let mut wine = Wine::from_binary(path.join("files/bin/wine64"))
            .with_arch(WineArch::Win64)
            .with_server(path.join("files/bin/wineserver"))
            .with_loader(WineLoader::Current);

        if let Some(prefix) = wine_prefix {
            wine = wine.with_prefix(prefix);
        }

        Self {
            wine,
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

    /// Inner function to update proton-related files
    fn update_proton_files(&self) -> anyhow::Result<()> {
        // This has to be Some unless library's user really knows what he does
        // in this case I'm nobody to stop him
        if let Some(path) = &self.proton_prefix {
            // Create `version` file in proton prefix based on `CURRENT_PREFIX_VERSION="..."` in `proton` script
            let mut found_version = false;

            if let Ok(proton) = std::fs::read_to_string(self.path.join("proton")) {
                if let Some(version) = proton.find("CURRENT_PREFIX_VERSION=\"") {
                    if let Some(version_end) = proton[version + 24..].find('"') {
                        let version = &proton[version + 24..version + 24 + version_end];

                        if !version.is_empty() {
                            std::fs::write(path.join("version"), version)?;

                            found_version = true;
                        }
                    }
                }
            }

            // If version wasn't found - just copy `version` file to proton prefix
            // Generally speaking I should try to parse correct version from this file
            // but GE-Proton dev messed up here in some builds (mistyped version as "GE=Proton..")
            // so I don't even try to do it here
            if !found_version && self.path.join("version").exists() {
                std::fs::copy(self.path.join("version"), path.join("version"))?;
            }

            // Copy `tracked_files` to proton prefix if this file exists
            if self.path.join("tracked_files").exists() {
                std::fs::copy(self.path.join("tracked_files"), path.join("tracked_files"))?;
            }

            // Otherwise try to find `proton_[version]_tracked_files` file in proton build folder
            else if let Ok(files) = std::fs::read_dir(&self.path) {
                for file in files.into_iter().flatten() {
                    let name = file.file_name();

                    // Minimal filename length requirements
                    if name.len() > 21 {
                        let name = name.to_string_lossy();

                        // Copy `tracked_files` to proton prefix
                        if &name[..7] == "proton_" && &name[name.len() - 14..] == "_tracked_files" {
                            std::fs::copy(file.path(), path.join("tracked_files"))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl WineWithExt for Proton {
    #[inline]
    /// Add path to proton directory
    /// 
    /// `prefix` should point to proton prefix, so wine prefix will be in `prefix/pfx`
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        let prefix = prefix.into();

        Self {
            wine: self.wine.with_prefix(prefix.join("pfx")),
            proton_prefix: Some(prefix),
            ..self
        }
    }

    #[inline]
    /// Add wine architecture
    fn with_arch(self, arch: WineArch) -> Self {
        Self {
            wine: self.wine.with_arch(arch),
            ..self
        }
    }

    #[inline]
    /// Add wineboot binary
    fn with_boot(self, boot: WineBoot) -> Self {
        Self {
            wine: self.wine.with_boot(boot),
            ..self
        }
    }

    #[inline]
    /// Add wineserver binary
    fn with_server<T: Into<PathBuf>>(self, server: T) -> Self {
        Self {
            wine: self.wine.with_server(server),
            ..self
        }
    }

    #[inline]
    /// Add wineloader binary
    fn with_loader(self, loader: WineLoader) -> Self {
        Self {
            wine: self.wine.with_loader(loader),
            ..self
        }
    }

    #[inline]
    /// Set wine shared libraries paths
    fn with_wine_libs(self, wine_libs: WineSharedLibs) -> Self {
        Self {
            wine: self.wine.with_wine_libs(wine_libs),
            ..self
        }
    }

    #[inline]
    /// Set gstreamer shared libraries paths
    fn with_gstreamer_libs(self, gstreamer_libs: GstreamerSharedLibs) -> Self {
        Self {
            wine: self.wine.with_gstreamer_libs(gstreamer_libs),
            ..self
        }
    }
}

impl WineBootExt for Proton {
    #[inline]
    /// Get base `wineboot` command. Will return `wine wineboot` if `self.wineboot()` is `None`
    fn wineboot_command(&self) -> Command {
        self.wine.wineboot_command()
    }

    #[inline]
    /// Initialize wine prefix
    /// 
    /// Runs `wineboot -i` command and creates `version`
    /// and `tracked_files` files in proton prefix
    fn init_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output> {
        let output = self.wine.init_prefix(path)?;

        self.update_proton_files()?;

        Ok(output)
    }

    #[inline]
    /// Update existing wine prefix
    /// 
    /// Runs `wineboot -u` command and creates `version`
    /// and `tracked_files` files in proton prefix
    fn update_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output> {
        let output = self.wine.update_prefix(path)?;

        self.update_proton_files()?;

        Ok(output)
    }

    #[inline]
    /// Stop running processes. Runs `wineboot -k` command, or `wineboot -f` if `force = true`
    fn stop_processes(&self, force: bool) -> anyhow::Result<Output> {
        self.wine.stop_processes(force)
    }

    #[inline]
    /// Imitate windows restart. Runs `wineboot -r` command
    fn restart(&self) -> anyhow::Result<Output> {
        self.wine.restart()
    }

    #[inline]
    /// Imitate windows shutdown. Runs `wineboot -s` command
    fn shutdown(&self) -> anyhow::Result<Output> {
        self.wine.shutdown()
    }

    #[inline]
    /// End wineboot session. Runs `wineboot -e` command
    fn end_session(&self) -> anyhow::Result<Output> {
        self.wine.end_session()
    }
}

impl WineRunExt for Proton {
    #[inline]
    /// Run the game using proton
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> anyhow::Result<Child> {
        self.run_args_with_env([binary], [])
    }

    #[inline]
    /// Run the game using proton
    /// 
    /// Note that it doesn't accept several arguments. You should use `[binary]` here only.
    /// This syntax remains here only because of `WineRunExt` trait
    fn run_args<T, S>(&self, args: T) -> anyhow::Result<Child>
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
    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        Ok(Command::new(self.python.as_os_str())
            .arg(self.path.join("proton"))
            .arg("run")
            .args(args)
            .envs(self.get_envs())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(envs)
            .spawn()?)
    }

    #[inline]
    fn winepath(&self, path: &str) -> anyhow::Result<PathBuf> {
        self.wine.winepath(path)
    }
}
