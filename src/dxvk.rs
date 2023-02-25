use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind, Result};

use derive_builder::Builder;

use super::wine::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    Win32,
    Win64
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct InstallParams {
    /// Install DXGI
    /// 
    /// Defualt is `true`
    pub dxgi: bool,

    /// Install D3D9
    /// 
    /// Defualt is `true`
    pub d3d9: bool,

    /// Install D3D10 Core
    /// 
    /// Defualt is `true`
    pub d3d10core: bool,

    /// Install D3D11
    /// 
    /// Defualt is `true`
    pub d3d11: bool,

    /// Ensure wine placeholder dlls are recreated if they are missing
    /// 
    /// Default is `true`
    pub repair_dlls: bool,

    /// Which library versions should be installed
    /// 
    /// Defualt is `Arch::Win64`
    pub arch: Arch
}

impl Default for InstallParams {
    fn default() -> Self {
        Self {
            dxgi: true,
            d3d9: true,
            d3d10core: true,
            d3d11: true,
            repair_dlls: true,
            arch: Arch::Win64
        }
    }
}

/// Add dll override to the wine prefix
pub fn install_dll(wine: &Wine, system32: &Path, dlls_folder: &Path, dll_name: &str) -> Result<()> {
    let src_path = dlls_folder.join(format!("{dll_name}.dll"));
    let dest_path = system32.join(format!("{dll_name}.dll"));
    let dest_path_old = system32.join(format!("{dll_name}.dll.old"));

    // Check dlls existence
    if !src_path.exists() {
        return Err(Error::new(ErrorKind::Other, "Failed to resolve path: ".to_string() + &src_path.to_string_lossy()));
    }

    if !dest_path.exists() {
        return Err(Error::new(ErrorKind::Other, "Failed to resolve path: ".to_string() + &dest_path.to_string_lossy()));
    }

    // Remove dest file (original one is already persisted)
    if dest_path_old.exists() {
        std::fs::remove_file(&dest_path)?;
    }

    // Rename dest file (it's (likely) an original one)
    else {
        std::fs::rename(&dest_path, dest_path_old)?;
    }

    // Copy dll to the destination location
    std::fs::copy(&src_path, &dest_path)?;

    // "$wine" reg add 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v $1 /d native /f
    let output = wine.run_args(["reg", "add", "HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides", "/v", dll_name, "/d", "native", "/f"])?.wait_with_output()?;

    match output.status.success() {
        true  => Ok(()),
        false => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            Err(Error::new(ErrorKind::Other, "Failed to add dll override: ".to_string() + stdout.trim_end().lines().last().unwrap_or(&stdout)))
        }
    }
}

/// Remove dll override from the wine prefix
pub fn restore_dll(wine: &Wine, system32: &Path, dll_name: &str) -> Result<()> {
    let dest_path = system32.join(format!("{dll_name}.dll"));
    let dest_path_old = system32.join(format!("{dll_name}.dll.old"));

    // Check dll existence
    if dest_path.exists() {
        // Original file exists so we'll restore it
        if dest_path_old.exists() {
            std::fs::remove_file(&dest_path)?;
            std::fs::rename(&dest_path_old, dest_path)?;
        }

        // Original file doesn't exist
        else {
            return Err(Error::new(ErrorKind::Other, "Failed to restore dll, original file is not persisted: ".to_string() + &dest_path.to_string_lossy()));
        }
    }

    // "$wine" reg delete 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v $1 /f
    let output = wine.run_args(["reg", "delete", "HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides", "/v", dll_name, "/f"])?.wait_with_output()?;

    match output.status.success() {
        true  => Ok(()),
        false => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            Err(Error::new(ErrorKind::Other, "Failed to add dll override: ".to_string() + stdout.trim_end().lines().last().unwrap_or(&stdout)))
        }
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
    pub fn get_version<T: Into<PathBuf>>(prefix: T) -> Result<Option<String>> {
        fn get_version(bytes: &[u8]) -> Option<String> {
            // 14 because [DXVK:] [\32] [\0] [v] [version number] [.] [version number] [.] [version number] [\0]
            // [version number] takes at least 1 byte so ..
            for i in 0..bytes.len() - 14 {
                if bytes[i]     == b'D' && bytes[i + 1] == b'X' && bytes[i + 2] == b'V' && bytes[i + 3] == b'K' &&
                   bytes[i + 4] == b':' && bytes[i + 5] == 32   && bytes[i + 6] == 0    && bytes[i + 7] == b'v'
                {
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

        let version = get_version(&bytes[offset_close_start..offset_close_end])    //           3 __ 4
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
    /// Dxvk::install(
    ///     &Wine::default(),
    ///     PathBuf::from("/path/to/dxvk-x.y.z"),
    ///     InstallParams::default()
    /// ).expect("Failed to install DXVK");
    /// ```
    pub fn install<T: Into<PathBuf>>(
        wine: &Wine,
        dxvk_folder: T,
        params: InstallParams
    ) -> Result<()> {
        match &wine.prefix {
            Some(prefix) => {
                // Check correctness of the wine prefix
                if !prefix.exists() || !prefix.join("system.reg").exists() {
                    return Err(Error::new(ErrorKind::Other, prefix.to_string_lossy() + " is not a valid wine prefix"));
                }

                // Verify and repair wine prefix if needed (and asked to)
                if params.repair_dlls {
                    let output = wine.update_prefix(prefix)?;

                    if !output.status.success() {
                        return Err(Error::new(ErrorKind::Other, "Failed to repair wine prefix: ".to_string() + &String::from_utf8_lossy(&output.stderr)));
                    }
                }

                let system32 = wine.winepath("C:\\windows\\system32")?;
                let dxvk_folder = dxvk_folder.into();

                // DXGI
                if params.dxgi {
                    match params.arch {
                        Arch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "dxgi")?,
                        Arch::Win64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "dxgi")?
                    }
                }

                // D3D9
                if params.d3d9 {
                    match params.arch {
                        Arch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "d3d9")?,
                        Arch::Win64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "d3d9")?
                    }
                }

                // D3D10 Core
                if params.d3d10core {
                    match params.arch {
                        Arch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "d3d10core")?,
                        Arch::Win64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "d3d10core")?
                    }
                }

                // D3D11
                if params.d3d11 {
                    match params.arch {
                        Arch::Win32 => install_dll(wine, &system32, &dxvk_folder.join("x32"), "d3d11")?,
                        Arch::Win64 => install_dll(wine, &system32, &dxvk_folder.join("x64"), "d3d11")?
                    }
                }

                Ok(())
            }

            None => Err(Error::new(ErrorKind::Other, "You must give a wine prefix path"))
        }
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
    ) -> Result<()> {
        match &wine.prefix {
            Some(prefix) => {
                // Check correctness of the wine prefix
                if !prefix.exists() || !prefix.join("system.reg").exists() {
                    return Err(Error::new(ErrorKind::Other, prefix.to_string_lossy() + " is not a valid wine prefix"));
                }

                // Verify and repair wine prefix if needed (and asked to)
                if params.repair_dlls {
                    let output = wine.update_prefix(prefix)?;

                    if !output.status.success() {
                        return Err(Error::new(ErrorKind::Other, "Failed to repair wine prefix: ".to_string() + &String::from_utf8_lossy(&output.stderr)));
                    }
                }

                let system32 = wine.winepath("C:\\windows\\system32")?;

                // DXGI
                if params.dxgi {
                    match params.arch {
                        Arch::Win32 => restore_dll(wine, &system32, "dxgi")?,
                        Arch::Win64 => restore_dll(wine, &system32, "dxgi")?
                    }
                }

                // D3D9
                if params.d3d9 {
                    match params.arch {
                        Arch::Win32 => restore_dll(wine, &system32, "d3d9")?,
                        Arch::Win64 => restore_dll(wine, &system32, "d3d9")?
                    }
                }

                // D3D10 Core
                if params.d3d10core {
                    match params.arch {
                        Arch::Win32 => restore_dll(wine, &system32, "d3d10core")?,
                        Arch::Win64 => restore_dll(wine, &system32, "d3d10core")?
                    }
                }

                // D3D11
                if params.d3d11 {
                    match params.arch {
                        Arch::Win32 => restore_dll(wine, &system32, "d3d11")?,
                        Arch::Win64 => restore_dll(wine, &system32, "d3d11")?
                    }
                }

                Ok(())
            }

            None => Err(Error::new(ErrorKind::Other, "You must give a wine prefix path"))
        }
    }
}
