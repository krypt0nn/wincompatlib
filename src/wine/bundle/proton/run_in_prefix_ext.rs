use std::process::{Command, Child, Stdio};
use std::ffi::OsStr;
use std::io::Result;

use super::Proton;

pub trait RunInPrefixExt {
    fn run_in_prefix<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child>;

    fn run_in_prefix_args<T, S>(&self, args: T) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn run_in_prefix_args_with_envs<T, K, S>(&self, args: T, envs: K) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>;
}

impl RunInPrefixExt for Proton {
    /// Executes `python3 proton runinprefix` command
    /// 
    /// In the end equal to:
    /// 
    /// ```bash
    /// ./proton/files/bin/wine binary
    /// ```
    #[inline]
    fn run_in_prefix<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child> {
        self.run_in_prefix_args_with_envs([binary], [])
    }

    /// Executes `python3 proton runinprefix` command
    /// 
    /// In the end equal to:
    /// 
    /// ```bash
    /// ./proton/files/bin/wine [args]
    /// ```
    #[inline]
    fn run_in_prefix_args<T, S>(&self, args: T) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.run_in_prefix_args_with_envs(args, [])
    }

    /// Executes `python3 proton runinprefix` command
    /// 
    /// In the end equal to:
    /// 
    /// ```bash
    /// ./proton/files/bin/wine [args]
    /// ```
    fn run_in_prefix_args_with_envs<T, K, S>(&self, args: T, envs: K) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        Command::new(self.python.as_os_str())
            .arg(self.path.join("proton"))
            .arg("runinprefix")
            .args(args)
            .envs(self.get_envs())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(envs)
            .spawn()
    }
}
