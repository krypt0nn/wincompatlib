use std::path::PathBuf;
use std::process::{Child, Command};

use super::*;

pub trait WineRunExt {
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child>;

    fn run_args<T, S>(&self, args: T) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>;

    fn winepath(&self, path: &str) -> Result<PathBuf>;
}

impl WineRunExt for Wine {
    /// Execute some command using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run("/your/executable");
    /// ```
    fn run<T: AsRef<OsStr>>(&self, binary: T) -> Result<Child> {
        self.run_args_with_env([binary], [])
    }

    /// Execute some command with args using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run_args(["/your/executable", "--help"]);
    /// ```
    fn run_args<T, S>(&self, args: T) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.run_args_with_env(args, [])
    }

    /// Execute some command with args and environment variables using wine
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// let process = Wine::default().run_args_with_env(["/your/executable", "--help"], [
    ///     ("YOUR", "variable")
    /// ]);
    /// ```
    fn run_args_with_env<T, K, S>(&self, args: T, envs: K) -> Result<Child>
    where
        T: IntoIterator<Item = S>,
        K: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>
    {
        let mut command = Command::new(&self.binary);

        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(prefix) = &self.prefix {
            command.env("WINEPREFIX", prefix);
        }

        if let Some(arch) = self.arch {
            command.env("WINEARCH", arch.to_str());
        }

        if let Some(server) = &self.wineserver {
            command.env("WINESERVER", server);
        }

        match &self.wineloader {
            WineLoader::Default => (),
            WineLoader::Current => {
                command.env("WINELOADER", &self.binary);
            },
            WineLoader::Custom(path) => {
                command.env("WINELOADER", path);
            }
        }

        command.envs(envs).spawn()
    }

    /// Get unix path to the windows folder in the wine prefix
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// println!("System32 path: {:?}", Wine::default().winepath("C:\\windows\\system32"));
    /// ```
    fn winepath(&self, path: &str) -> Result<PathBuf> {
        let output = self.run_args(["winepath", "-u", path])?.wait_with_output()?;

        match output.status.success() {
            true => {
                // It adds "\n" in the end which is 1 byte long
                let path = PathBuf::from(OsString::from_vec(output.stdout[..output.stdout.len() - 1].to_vec()));

                match path.exists() {
                    true  => Ok(path),
                    false => Err(Error::new(ErrorKind::Other, "Wine path is not correct: ".to_string() + &String::from_utf8_lossy(&output.stdout)))
                }
            }

            false => Err(Error::new(ErrorKind::Other, "Failed to find wine path: ".to_string() + &String::from_utf8_lossy(&output.stdout)))
        }
    }
}
