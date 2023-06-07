use std::path::PathBuf;
use std::process::{Child, Command};
use std::ffi::OsStr;

use crate::wine::*;

pub trait WineRunExt {
    /// Execute some command using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run("/your/executable");
    /// ```
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> anyhow::Result<Child>;

    /// Execute some command with args using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run_args(["/your/executable", "--help"]);
    /// ```
    fn run_args<T, S>(&self, args: T) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    /// Execute some command with args and environment variables using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run_args_with_env(["/your/executable", "--help"], [
    ///     ("YOUR", "variable")
    /// ]);
    /// ```
    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>;

    /// Get unix path to the windows folder in the wine prefix
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// println!("System32 path: {:?}", Wine::default().winepath("C:\\windows\\system32"));
    /// ```
    fn winepath(&self, path: &str) -> anyhow::Result<PathBuf>;
}

impl WineRunExt for Wine {
    #[inline]
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> anyhow::Result<Child> {
        self.run_args_with_env([binary], [])
    }

    #[inline]
    fn run_args<T, S>(&self, args: T) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.run_args_with_env(args, [])
    }

    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> anyhow::Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        Ok(Command::new(&self.binary)
            .args(args)
            .envs(self.get_envs())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(envs)
            .spawn()?)
    }

    fn winepath(&self, path: &str) -> anyhow::Result<PathBuf> {
        let output = self.run_args(["winepath", "-u", path])?.wait_with_output()?;

        let true = output.status.success() else {
            anyhow::bail!("Failed to find wine path: {}", String::from_utf8_lossy(&output.stdout));
        };

        // It adds "\n" in the end which is 1 byte long
        let path = PathBuf::from(OsString::from_vec(output.stdout[..output.stdout.len() - 1].to_vec()));

        if !path.exists() {
            anyhow::bail!("Wine path is not correct: {}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(path)
    }
}
