use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};

use super::wine::*;
use super::wine::ext::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallParams {
    /// Install DXGI
    ///
    /// Default is `true`
    pub dxgi: bool,

    /// Install D3D9
    ///
    /// Default is `true`
    pub d3d9: bool,

    /// Install D3D10 Core
    ///
    /// Default is `true`
    pub d3d10core: bool,

    /// Install D3D11
    ///
    /// Default is `true`
    pub d3d11: bool,

    /// Ensure wine placeholder dlls are recreated if they are missing
    ///
    /// Default is `true`
    pub repair_dlls: bool,

    /// Which library versions should be installed
    ///
    /// Default is `WineArch::Win64`
    pub arch: WineArch
}

impl Default for InstallParams {
    fn default() -> Self {
        Self {
            dxgi: true,
            d3d9: true,
            d3d10core: true,
            d3d11: true,
            repair_dlls: true,
            arch: WineArch::default()
        }
    }
}

/// Add dll override to the wine prefix
pub fn install_dll(wine: &Wine, system32: &Path, dlls_folder: &Path, dll_name: &str) -> anyhow::Result<()> {
    let src_path = dlls_folder.join(format!("{dll_name}.dll"));
    let dest_path = system32.join(format!("{dll_name}.dll"));
    let dest_path_old = system32.join(format!("{dll_name}.dll.old"));

    // Check dlls existence
    if !src_path.exists() {
        anyhow::bail!("Source path doesn't exist: {:?}", src_path);
    }

    if !dest_path.exists() {
        anyhow::bail!("Destination path doesn't exist: {:?}", dest_path);
    }

    // Remove dest file (original one is already persisted)
    if dest_path_old.exists() {
        std::fs::remove_file(&dest_path)?;
    }

    // Rename dest file (it's (likely) an original one)
    else {
        std::fs::rename(&dest_path, &dest_path_old)?;
    }

    // Copy dll to the destination location
    std::fs::copy(&src_path, &dest_path)?;

    // Try to add override and return original file back if we failed
    if let Err(err) = wine.add_override(dll_name, [OverrideMode::Native]) {
        std::fs::remove_file(&dest_path)?;
        std::fs::rename(&dest_path_old, &dest_path)?;

        anyhow::bail!(err);
    }

    Ok(())
}

/// Remove dll override from the wine prefix
pub fn restore_dll(wine: &Wine, system32: &Path, dll_name: &str) -> anyhow::Result<()> {
    let dest_path = system32.join(format!("{dll_name}.dll"));
    let dest_path_old = system32.join(format!("{dll_name}.dll.old"));

    // Original file exists so we'll restore it
    if dest_path_old.exists() {
        wine.delete_override(dll_name)?;

        if dest_path.exists() {
            std::fs::remove_file(&dest_path)?;
        }

        std::fs::rename(&dest_path_old, dest_path)?;

        Ok(())
    }

    // Original file doesn't exist
    else {
        anyhow::bail!("Failed to restore dll, original file doesn't exist: {:?}", dest_path_old);
    }
}

pub struct Dxvk;

impl Dxvk {
    /// Try to get applied DXVK version from the prefix path
    ///
    /// Returns:
    /// 1) `Ok(Some(..))` if version was found
    /// 2) `Ok(None)` if version wasn't found, so dxvk is not applied
    /// 3) `Err(..)` if failed to get applied dxvk version, likely because wrong prefix path specified
    ///
    /// ```
    /// use wincompatlib::prelude::*;
    ///
    /// match Dxvk::get_version("/path/to/prefix") {
    ///     Ok(Some(version)) => println!("DXVK applied: {}", version),
    ///     Ok(None) => println!("DXVK is not applied"),
    ///     Err(err) => eprintln!("Failed to get DXVK version: {}", err)
    /// }
    /// ```
    pub fn get_version<T: Into<PathBuf>>(prefix: T) -> anyhow::Result<Option<String>> {
        fn get_version(bytes: &[u8]) -> Option<String> {
            // 14 because [DXVK:] [\32] [\0] [v] [version number] [.] [version number] [.] [version number] [\0]
            // [version number] takes at least 1 byte so ..
            for i in 0..bytes.len() - 14 {
                if bytes[i..=i + 7] == [b'D', b'X', b'V', b'K', b':', 32, 0, b'v'] {
                    let mut version = String::new();

                    for byte in bytes.iter().skip(i + 8) {
                        if *byte != 0 {
                            version.push((*byte).into());
                        }

                        else {
                            break;
                        }
                    }

                    return Some(version);
                }
            }

            None
        }

        let prefix: PathBuf = prefix.into();

        // [DXVK:] hints offsets in 2.1 (~)
        // d3d11: 2789063
        //  dxgi: 1881252
        //
        // We'll try to find the version sequence starting from closest approximated address,
        // then extending this sequence in both directions untill we reach whole file size
        //
        // Bytes sequence:
        //
        // 1       2   3 4   5       6
        // [       [   [ ]   ]       ]
        //             ^ offset_close_start
        //               ^ offset_close_end
        //         ^ offset_wide_start
        //                   ^ offset_wide_end
        // ^ start
        //                           ^ end

        let offset_close_start;
        let offset_close_end;

        let offset_wide_start;
        let offset_wide_end;

        let bytes = match std::fs::read(prefix.join("drive_c/windows/system32/d3d11.dll")) {
            Ok(bytes) => {
                offset_close_start = 2500000;
                offset_close_end   = 2900000;

                offset_wide_start = 2000000;
                offset_wide_end   = 3200000;

                bytes
            }

            Err(_) => {
                offset_close_start = 1600000;
                offset_close_end   = 2000000;

                offset_wide_start = 1000000;
                offset_wide_end   = 2300000;

                std::fs::read(prefix.join("drive_c/windows/system32/dxgi.dll"))?
            }
        };

        if bytes.len() < offset_wide_end {
            return Ok(get_version(&bytes));
        }

        let version = get_version(&bytes[offset_close_start..offset_close_end])            //           3 __ 4
            .unwrap_or_else(|| get_version(&bytes[offset_wide_start..offset_close_start])  //      2 __ 3    |
            .unwrap_or_else(|| get_version(&bytes[offset_close_end..offset_wide_end])      //      |         4 __ 5
            .unwrap_or_else(|| get_version(&bytes[..offset_wide_start])                    // 1 __ 2              |
            .unwrap_or_else(|| get_version(&bytes[offset_wide_end..])                      //                     5 __ 6
            .unwrap_or_default()))));

        if version.is_empty() {
            Ok(None)
        } else {
            Ok(Some(version))
        }
    }

    /// Install DXVK to wine prefix
    ///
    /// ```no_run
    /// use wincompatlib::prelude::*;
    ///
    /// use std::path::PathBuf;
    ///
    /// Dxvk::install(Wine::default(), "/path/to/dxvk-x.y.z", InstallParams::default())
    ///     .expect("Failed to install DXVK");
    /// ```
    pub fn install(
        wine: impl AsRef<Wine>,
        dxvk_folder: impl Into<PathBuf>,
        params: InstallParams
    ) -> anyhow::Result<()> {
        let wine = wine.as_ref();

        // Check correctness of the wine prefix
        if !wine.prefix.exists() || !wine.prefix.join("system.reg").exists() {
            anyhow::bail!("{:?} is not a valid wine prefix", wine.prefix);
        }

        // Verify and repair wine prefix if needed (and asked to)
        if params.repair_dlls {
            let output = wine.update_prefix(None::<&str>)?;

            if !output.status.success() {
                anyhow::bail!("Failed to repair wine prefix: {}", String::from_utf8_lossy(&output.stderr));
            }
        }

        let system32 = wine.winepath("C:\\windows\\system32")?;
        let dxvk_folder = dxvk_folder.into();

        // DXGI
        if params.dxgi {
            match params.arch {
                WineArch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "dxgi")?,
                WineArch::Win64 | WineArch::Wow64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "dxgi")?
            }
        }

        // D3D9
        if params.d3d9 {
            match params.arch {
                WineArch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "d3d9")?,
                WineArch::Win64 | WineArch::Wow64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "d3d9")?
            }
        }

        // D3D10 Core
        if params.d3d10core {
            match params.arch {
                WineArch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "d3d10core")?,
                WineArch::Win64 | WineArch::Wow64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "d3d10core")?
            }
        }

        // D3D11
        if params.d3d11 {
            match params.arch {
                WineArch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "d3d11")?,
                WineArch::Win64 | WineArch::Wow64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "d3d11")?
            }
        }

        Ok(())
    }

    /// Uninstall DXVK from wine prefix
    ///
    /// ```no_run
    /// use wincompatlib::prelude::*;
    ///
    /// use std::path::PathBuf;
    ///
    /// Dxvk::uninstall(
    ///     &Wine::default(),
    ///     InstallParams::default()
    /// ).expect("Failed to uninstall DXVK");
    /// ```
    pub fn uninstall(
        wine: &Wine,
        params: InstallParams
    ) -> anyhow::Result<()> {
        // Check correctness of the wine prefix
        if !wine.prefix.exists() || !wine.prefix.join("system.reg").exists() {
            anyhow::bail!("{:?} is not a valid wine prefix", wine.prefix);
        }

        // Verify and repair wine prefix if needed (and asked to)
        if params.repair_dlls {
            let output = wine.update_prefix(None::<&str>)?;

            if !output.status.success() {
                anyhow::bail!("Failed to repair wine prefix: {}", String::from_utf8_lossy(&output.stderr));
            }
        }

        let system32 = wine.winepath("C:\\windows\\system32")?;

        // DXGI
        if params.dxgi {
            match params.arch {
                WineArch::Win32 => restore_dll(wine, &system32, "dxgi")?,
                WineArch::Win64 | WineArch::Wow64 => restore_dll(wine, &system32, "dxgi")?
            }
        }

        // D3D9
        if params.d3d9 {
            match params.arch {
                WineArch::Win32 => restore_dll(wine, &system32, "d3d9")?,
                WineArch::Win64 | WineArch::Wow64 => restore_dll(wine, &system32, "d3d9")?
            }
        }

        // D3D10 Core
        if params.d3d10core {
            match params.arch {
                WineArch::Win32 => restore_dll(wine, &system32, "d3d10core")?,
                WineArch::Win64 | WineArch::Wow64 => restore_dll(wine, &system32, "d3d10core")?
            }
        }

        // D3D11
        if params.d3d11 {
            match params.arch {
                WineArch::Win32 => restore_dll(wine, &system32, "d3d11")?,
                WineArch::Win64 | WineArch::Wow64 => restore_dll(wine, &system32, "d3d11")?
            }
        }

        Ok(())
    }
}
