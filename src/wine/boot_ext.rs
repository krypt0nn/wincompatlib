use std::path::PathBuf;

use super::*;

pub trait WineBootExt {
    fn update_prefix<T: Into<PathBuf>>(&self, path: T) -> Result<Output>;
    fn stop_processes(&self, force: bool) -> Result<Output>;
    fn restart(&self) -> Result<Output>;
    fn shutdown(&self) -> Result<Output>;
    fn end_session(&self) -> Result<Output>;
}

impl WineBootExt for Wine {
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
        Command::new(self.wineboot())
            .arg("-u")
            .envs(self.get_envs())
            .env("WINEPREFIX", path.into())
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
        Command::new(self.wineboot())
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
        Command::new(self.wineboot())
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
        Command::new(self.wineboot())
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
        Command::new(self.wineboot())
            .arg("-e")
            .envs(self.get_envs())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }
}
