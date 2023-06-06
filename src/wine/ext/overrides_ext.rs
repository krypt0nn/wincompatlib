use std::io::{Error, ErrorKind, Result};

use crate::wine::*;
use crate::wine::ext::WineRunExt;

pub trait WineOverridesExt {
    /// Add dll override to the wine registry
    fn add_override(&self, dll_name: impl AsRef<str>) -> Result<()>;

    /// Remove dll override from the wine registry
    fn delete_override(&self, dll_name: impl AsRef<str>) -> Result<()>;
}

impl WineOverridesExt for Wine {
    fn add_override(&self, dll_name: impl AsRef<str>) -> Result<()> {
        // "$wine" reg add 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v $1 /d native /f
        let output = self.run_args(["reg", "add", "HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides", "/v", dll_name.as_ref(), "/d", "native", "/f"])?
            .wait_with_output()?;

        if output.status.success() {
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let error = stdout.trim_end().lines().last().unwrap_or(&stdout);

        Err(Error::new(ErrorKind::Other, format!("Failed to add dll override: {error}")))
    }

    fn delete_override(&self, dll_name: impl AsRef<str>) -> Result<()> {
        // "$wine" reg delete 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v $1 /f
        let output = self.run_args(["reg", "delete", "HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides", "/v", dll_name.as_ref(), "/f"])?
            .wait_with_output()?;

        if output.status.success() {
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let error = stdout.trim_end().lines().last().unwrap_or(&stdout);

        Err(Error::new(ErrorKind::Other, format!("Failed to remove dll override: {error}")))
    }
}
