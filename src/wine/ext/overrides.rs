use crate::wine::*;
use crate::wine::ext::WineRunExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Some info can be found here:
/// 
/// https://wiki.winehq.org/Wine_User%27s_Guide#DLL_Overrides
pub enum OverrideMode {
    Native,
    Builtin,
    Disabled
}

impl OverrideMode {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Native   => "native",
            Self::Builtin  => "builtin",
            Self::Disabled => "disabled"
        }
    }
}

// TODO: modify user.reg / system.reg manually instead of calling reg.exe

pub trait WineOverridesExt {
    /// Add dll override to the wine registry
    fn add_override(&self, dll_name: impl AsRef<str>, modes: impl IntoIterator<Item = OverrideMode>) -> anyhow::Result<()>;

    /// Remove dll override from the wine registry
    fn delete_override(&self, dll_name: impl AsRef<str>) -> anyhow::Result<()>;
}

impl WineOverridesExt for Wine {
    fn add_override(&self, dll_name: impl AsRef<str>, modes: impl IntoIterator<Item = OverrideMode>) -> anyhow::Result<()> {
        let modes = modes.into_iter()
            .map(|mode| mode.to_str())
            .collect::<Vec<&'static str>>()
            .join(",");

        // "$wine" reg add 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v $1 /d native /f
        let output = self.run_args(["reg", "add", "HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides", "/v", dll_name.as_ref(), "/d", &modes, "/f"])?
            .wait_with_output()?;

        if output.status.success() {
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let error = stdout.trim_end().lines().last().unwrap_or(&stdout);

        anyhow::bail!("Failed to add dll override: {error}");
    }

    fn delete_override(&self, dll_name: impl AsRef<str>) -> anyhow::Result<()> {
        // "$wine" reg delete 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v $1 /f
        let output = self.run_args(["reg", "delete", "HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides", "/v", dll_name.as_ref(), "/f"])?
            .wait_with_output()?;

        if output.status.success() {
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let error = stdout.trim_end().lines().last().unwrap_or(&stdout);

        anyhow::bail!("Failed to remove dll override: {error}");
    }
}
