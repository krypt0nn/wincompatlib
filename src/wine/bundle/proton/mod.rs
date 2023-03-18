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
    /// `prefix` should point to proton prefix, so wine prefix will be in `prefix/pfx`
    #[inline]
    fn with_prefix<T: Into<PathBuf>>(self, prefix: T) -> Self {
        let prefix = prefix.into();

        Self {
            wine: self.wine.with_prefix(prefix.join("pfx")),
            proton_prefix: Some(prefix),
            ..self
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

    /// Create (or update existing) wine prefix.
    /// Runs `wineboot -u` command and creates `version` and `tracked_files` files
    /// in proton prefix
    fn update_prefix<T: Into<PathBuf>>(&self, path: Option<T>) -> Result<Output> {
        // Update wine prefix
        let output = self.wine.update_prefix(path)?;

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

        Ok(output)
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
