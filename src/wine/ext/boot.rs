use std::process::Output;

use crate::wine::*;

pub trait WineBootExt {
    /// Get base `wineboot` command. Will return `wine wineboot` if `self.wineboot()` is `None`
    fn wineboot_command(&self) -> Command;

    /// Initialize wine prefix. Runs `wineboot -i` command
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .init_prefix(Some("/path/to/prefix"))
    ///     .expect("Failed to create prefix");
    /// ```
    /// 
    /// Use prefix specified in current wine struct:
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .with_prefix("/path/to/prefix")
    ///     .init_prefix(None::<&str>) // Don't even ask
    ///     .expect("Failed to create prefix");
    /// ```
    /// 
    /// If prefix is not specified in `Wine` struct and is not given to `update_prefix` method -
    /// then `Err` will be returned
    fn init_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output>;

    /// Update existing wine prefix. Runs `wineboot -u` command
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .update_prefix(Some("/path/to/prefix"))
    ///     .expect("Failed to update prefix");
    /// ```
    /// 
    /// Use prefix specified in current wine struct:
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .with_prefix("/path/to/prefix")
    ///     .update_prefix(None::<&str>) // Don't even ask
    ///     .expect("Failed to update prefix");
    /// ```
    /// 
    /// If prefix is not specified in `Wine` struct and is not given to `update_prefix` method -
    /// then `Err` will be returned
    fn update_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output>;

    /// Stop running processes. Runs `wineboot -k` command, or `wineboot -f` if `force = true`
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .stop_processes(false)
    ///     .expect("Failed to update prefix");
    /// ```
    fn stop_processes(&self, force: bool) -> anyhow::Result<Output>;

    /// Imitate windows restart. Runs `wineboot -r` command
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .with_prefix("/path/to/prefix")
    ///     .restart()
    ///     .expect("Failed to restart");
    /// ```
    fn restart(&self) -> anyhow::Result<Output>;

    /// Imitate windows shutdown. Runs `wineboot -s` command
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .with_prefix("/path/to/prefix")
    ///     .shutdown()
    ///     .expect("Failed to shutdown");
    /// ```
    fn shutdown(&self) -> anyhow::Result<Output>;

    /// End wineboot session. Runs `wineboot -e` command
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .with_prefix("/path/to/prefix")
    ///     .end_session()
    ///     .expect("Failed to shutdown");
    /// ```
    fn end_session(&self) -> anyhow::Result<Output>;
}

impl WineBootExt for Wine {
    fn wineboot_command(&self) -> Command {
        match self.wineboot() {
            Some(WineBoot::Unix(wineboot)) => Command::new(wineboot),

            Some(WineBoot::Windows(wineboot)) => {
                let mut command = Command::new(&self.binary);

                command.arg(wineboot);

                command
            }

            None => {
                let mut command = Command::new(&self.binary);

                command.arg("wineboot");

                command
            }
        }
    }

    fn init_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output> {
        let path = match path {
            Some(path) => path.into(),
            None => self.prefix.to_owned()
        };

        // Create all parent directories
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        Ok(self.wineboot_command()
            .arg("-i")
            .envs(self.get_envs())
            .env("WINEPREFIX", path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?)
    }

    fn update_prefix(&self, path: Option<impl Into<PathBuf>>) -> anyhow::Result<Output> {
        let path = match path {
            Some(path) => path.into(),
            None => self.prefix.to_owned()
        };

        // Create all parent directories
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        Ok(self.wineboot_command()
            .arg("-u")
            .envs(self.get_envs())
            .env("WINEPREFIX", path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?)
    }

    fn stop_processes(&self, force: bool) -> anyhow::Result<Output> {
        Ok(self.wineboot_command()
            .arg(if force { "-f" } else { "-k" })
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?)
    }

    fn restart(&self) -> anyhow::Result<Output> {
        Ok(self.wineboot_command()
            .arg("-r")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?)
    }

    fn shutdown(&self) -> anyhow::Result<Output> {
        Ok(self.wineboot_command()
            .arg("-s")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?)
    }

    fn end_session(&self) -> anyhow::Result<Output> {
        Ok(self.wineboot_command()
            .arg("-e")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?)
    }
}
