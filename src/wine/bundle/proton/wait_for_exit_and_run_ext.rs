use std::process::{Command, Child, Stdio};
use std::ffi::OsStr;
use std::io::Result;

use super::Proton;

pub trait WaitForExitAndRunExt {
    /// Executes `python3 proton waitforexitandrun` command
    /// 
    /// In the end equal to:
    /// 
    /// ```bash
    /// ./proton/files/bin/wineserver -w
    /// ./proton/files/bin/wine64 'c:\windows\system32\steam.exe' binary
    /// ```
    fn wait_for_exit_and_run<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child>;

    /// Executes `python3 proton waitforexitandrun` command
    /// 
    /// In the end equal to:
    /// 
    /// ```bash
    /// ./proton/files/bin/wineserver -w
    /// ./proton/files/bin/wine64 'c:\windows\system32\steam.exe' binary
    /// ```
    fn wait_for_exit_and_run_with_envs<T, S>(&self, binary: T, envs: S) -> Result<Child>
    where
        T: AsRef<OsStr>,
        S: IntoIterator<Item = (T, T)>;
}

impl WaitForExitAndRunExt for Proton {
    #[inline]
    fn wait_for_exit_and_run<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child> {
        self.wait_for_exit_and_run_with_envs(binary, [])
    }

    fn wait_for_exit_and_run_with_envs<T, S>(&self, binary: T, envs: S) -> Result<Child>
    where
        T: AsRef<OsStr>,
        S: IntoIterator<Item = (T, T)>
    {
        Command::new(self.python.as_os_str())
            .arg(self.path.join("proton"))
            .arg("waitforexitandrun")
            .arg(binary)
            .envs(self.get_envs())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(envs)
            .spawn()
    }
}
