use std::path::PathBuf;

use super::*;

pub trait WineBootExt {
    fn wineboot_command(&self) -> Command;
    fn update_prefix<T: Into<PathBuf>>(&self, path: T) -> Result<Output>;
    fn stop_processes(&self, force: bool) -> Result<Output>;
    fn restart(&self) -> Result<Output>;
    fn shutdown(&self) -> Result<Output>;
    fn end_session(&self) -> Result<Output>;
}

impl WineBootExt for Wine {
    /// Get base `wineboot` command. Will return `wine wineboot` if `self.wineboot()` is `None`
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

    /// Create (or update existing) wine prefix. Runs `wineboot -u` command
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .update_prefix("/path/to/prefix")
    ///     .expect("Failed to update prefix");
    /// ```
    fn update_prefix<T: Into<PathBuf>>(&self, path: T) -> Result<Output> {
        let path = path.into();

        // Create all parent directories
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        self.wineboot_command()
            .arg("-u")
            .envs(self.get_envs())
            .env("WINEPREFIX", path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

    /// Stop running processes. Runs `wineboot -k` command, or `wineboot -f` if `force = true`
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// Wine::default()
    ///     .stop_processes(false)
    ///     .expect("Failed to update prefix");
    /// ```
    fn stop_processes(&self, force: bool) -> Result<Output> {
        self.wineboot_command()
            .arg(if force { "-f" } else { "-k" })
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

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
    fn restart(&self) -> Result<Output> {
        self.wineboot_command()
            .arg("-r")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

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
    fn shutdown(&self) -> Result<Output> {
        self.wineboot_command()
            .arg("-s")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

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
    fn end_session(&self) -> Result<Output> {
        self.wineboot_command()
            .arg("-e")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }
}
