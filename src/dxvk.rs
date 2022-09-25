use std::path::PathBuf;
use std::io::{Error, ErrorKind, Result};
use std::process::{Command, Output};

use regex::Regex;

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
        let prefix: PathBuf = prefix.into();

        let (bytes, from, to) = match std::fs::read(prefix.join("drive_c/windows/system32/dxgi.dll")) {
            Ok(bytes) => (bytes, 1600000, 1700000),
            Err(_) => {
                let bytes = std::fs::read(prefix.join("drive_c/windows/system32/d3d11.dll"))?;

                (bytes, 2400000, 2500000)
            }
        };

        let bytes = if bytes.len() > to {
            bytes[from..to].to_vec()
        } else {
            return Ok(None);
        };

        // 8 because [\0] [v] [version number] [.] [version number] [.] [version number] [\0]
        // [version number] takes at least 1 byte so ..
        for i in 0..bytes.len() - 8 {
            if bytes[i] == b'\0' && bytes[i + 1] == b'v' {
                let mut version = [String::new(), String::new(), String::new()];
                let mut pointer = 0;

                for j in i + 2..bytes.len() - 6 {
                    if bytes[j] == b'.' {
                        pointer += 1;

                        if pointer > 2 {
                            break;
                        }
                    }

                    else if bytes[j] >= b'0' && bytes[j] <= b'9' {
                        version[pointer] += match bytes[j] {
                            b'0' => "0",
                            b'1' => "1",
                            b'2' => "2",
                            b'3' => "3",
                            b'4' => "4",
                            b'5' => "5",
                            b'6' => "6",
                            b'7' => "7",
                            b'8' => "8",
                            b'9' => "9",
                            _ => ""
                        };
                    }

                    else {
                        break;
                    }
                }

                if pointer == 2 && !version[0].is_empty() && !version[1].is_empty() && !version[2].is_empty() {
                    return Ok(Some(format!("{}.{}.{}", version[0], version[1], version[2])));
                }
            }
        }

        Ok(None)
    }

    fn prepare_script(
        setup_script: PathBuf,
        wine_path: PathBuf,
        wine64_path: PathBuf,
        wineboot_path: PathBuf
    ) -> Result<()> {
        let setup_script_path = setup_script;
        let mut setup_script = std::fs::read_to_string(&setup_script_path)?;

        let wine = Regex::new("wine=\".*\"").unwrap();
        let wine64 = Regex::new("wine64=\".*\"").unwrap();
        let wineboot = Regex::new("wineboot=\".*\"").unwrap();

        // Update wine paths
        setup_script = wine.replace_all(&setup_script, &format!("wine={:?}", wine_path)).to_string();
        setup_script = wine64.replace_all(&setup_script, &format!("wine64={:?}", wine64_path)).to_string();
        setup_script = wineboot.replace_all(&setup_script, &format!("wineboot={:?}", wineboot_path)).to_string();

        // Use wine64 to update wine prefix instead of running wineboot
        // so we can get rid of 32bit support
        setup_script = setup_script.replace("$wineboot -u", "\"$wine64\" -u");

        // Fix issues related to spaces in paths to the runners folder
        setup_script = setup_script.replace("which $wineboot", "which \"$wineboot\"");
        setup_script = setup_script.replace("$wine --version", "\"$wine\" --version");
        setup_script = setup_script.replace("$wine64 winepath", "\"$wine64\" winepath");
        setup_script = setup_script.replace("$wine winepath", "\"$wine\" winepath");
        setup_script = setup_script.replace("$wine reg", "\"$wine\" reg");

        // Old GE builds return specific --version output which can break
        // DXVK installation script
        setup_script = setup_script.replace("grep wine", "grep \"wine\\|GE\"");

        std::fs::write(&setup_script_path, setup_script)?;

        Ok(())
    }

    /// Install DXVK to wine prefix
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// use std::path::PathBuf;
    /// 
    /// let output = Dxvk::install(
    ///     PathBuf::from("/path/to/setup_dxvk.sh"),
    ///     PathBuf::from("/path/to/wine/prefix"),
    ///     PathBuf::from("/path/to/wine"),
    ///     PathBuf::from("/path/to/wine64"),
    ///     PathBuf::from("/path/to/wineboot"),
    ///     PathBuf::from("/path/to/wineserver")
    /// ).expect("Failed to install DXVK");
    /// 
    /// println!("Installing output: {}", String::from_utf8_lossy(&output.stdout));
    /// ```
    pub fn install(
        setup_script: PathBuf,
        prefix_path: PathBuf,
        wine_path: PathBuf,
        wine64_path: PathBuf,
        wineboot_path: PathBuf,
        wineserver_path: PathBuf
    ) -> Result<Output> {
        let setup_script_path = setup_script.clone();

        Self::prepare_script(
            setup_script,
            wine_path,
            wine64_path,
            wineboot_path
        )?;

        let output = Command::new("bash")
            .arg(&setup_script_path)
            .arg("install")
            .env("WINEPREFIX", prefix_path)
            .env("WINESERVER", wineserver_path)
            .output()?;

        if output.status.success() {
            Ok(output)
        }

        else {
            Err(Error::new(ErrorKind::Other, String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Uninstall DXVK from wine prefix
    /// 
    /// ```no_run
    /// use wincompatlib::prelude::*;
    /// 
    /// use std::path::PathBuf;
    /// 
    /// let output = Dxvk::uninstall(
    ///     PathBuf::from("/path/to/setup_dxvk.sh"),
    ///     PathBuf::from("/path/to/wine/prefix"),
    ///     PathBuf::from("/path/to/wine"),
    ///     PathBuf::from("/path/to/wine64"),
    ///     PathBuf::from("/path/to/wineboot"),
    ///     PathBuf::from("/path/to/wineserver")
    /// ).expect("Failed to uninstall DXVK");
    /// 
    /// println!("Uninstalling output: {}", String::from_utf8_lossy(&output.stdout));
    /// ```
    pub fn uninstall(
        setup_script: PathBuf,
        prefix_path: PathBuf,
        wine_path: PathBuf,
        wine64_path: PathBuf,
        wineboot_path: PathBuf,
        wineserver_path: PathBuf
    ) -> Result<Output> {
        let setup_script_path = setup_script.clone();

        Self::prepare_script(
            setup_script,
            wine_path,
            wine64_path,
            wineboot_path
        )?;

        let output = Command::new("bash")
            .arg(&setup_script_path)
            .arg("uninstall")
            .env("WINEPREFIX", prefix_path)
            .env("WINESERVER", wineserver_path)
            .output()?;

        if output.status.success() {
            Ok(output)
        }

        else {
            Err(Error::new(ErrorKind::Other, String::from_utf8_lossy(&output.stderr)))
        }
    }
}
