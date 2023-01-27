use std::ffi::OsString;
use std::process::Command;
use std::path::PathBuf;
use std::str::FromStr;

use serial_test::*;

use crate::prelude::*;

use crate::wine::WineRunExt;

fn get_test_dir() -> PathBuf {
    std::env::temp_dir().join("wincompatlib-test")
}

fn get_prefix_dir() -> PathBuf {
    get_test_dir().join("prefix")
}

fn get_custom_wine() -> Wine {
    let test_dir = get_test_dir();

    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)
            .expect("Failed to create test directory");
    }

    let wine_dir = test_dir.join("lutris-GE-Proton7-29-x86_64");

    if !wine_dir.exists() {
        Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg("https://github.com/GloriousEggroll/wine-ge-custom/releases/download/GE-Proton7-29/wine-lutris-GE-Proton7-29-x86_64.tar.xz")
            .arg("-o")
            .arg(test_dir.join("wine.tar.xz"))
            .output()
            .expect("Failed to download wine. Curl is not available?");

        Command::new("tar")
            .arg("-xf")
            .arg("wine.tar.xz")
            .current_dir(test_dir)
            .output()
            .expect("Failed to extract downloaded wine. Tar is not available?");
    }

    Wine::from_binary(wine_dir.join("bin/wine64"))
        .with_loader(WineLoader::Current)
        .with_arch(WineArch::Win64)
}

#[cfg(feature = "dxvk")]
fn get_dxvk_folder() -> PathBuf {
    let test_dir = get_test_dir();

    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)
            .expect("Failed to create test directory");
    }

    let dxvk_dir = test_dir.join("dxvk-2.1");

    if !dxvk_dir.exists() {
        Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg("https://github.com/doitsujin/dxvk/releases/download/v2.1/dxvk-2.1.tar.gz")
            .arg("-o")
            .arg(test_dir.join("dxvk.tar.gz"))
            .output()
            .expect("Failed to download dxvk. Curl is not available?");

        Command::new("tar")
            .arg("-xf")
            .arg("dxvk.tar.gz")
            .current_dir(test_dir)
            .output()
            .expect("Failed to extract downloaded dxvk. Tar is not available?");
    }

    dxvk_dir
}

#[test]
#[parallel]
fn wine_version() {
    assert!(Wine::from_binary("\0").version().is_err());

    let version = get_custom_wine().version();

    assert!(version.is_ok());
    assert_eq!(version.unwrap(), OsString::from_str("wine-5.12-15203-g5a125f26458 (Staging)\n").unwrap());
}

#[test]
#[serial]
#[cfg(feature = "dxvk")]
fn apply_dxvk() -> std::io::Result<()> {
    // Test non existing prefix version
    assert!(Dxvk::get_version("\0").is_err());

    // Test DXVK uninstalling
    let dxvk_folder = get_dxvk_folder();
    let wine = get_custom_wine().with_prefix(get_prefix_dir());

    #[allow(unused_must_use)]
    {
        wine.uninstall_dxvk(InstallParams::default());
    }

    // Test clear prefix DXVK version
    let version = Dxvk::get_version(get_prefix_dir());

    assert_eq!(version.unwrap(), None);

    // Test DXVK installing
    wine.install_dxvk(dxvk_folder, InstallParams::default())?;

    // Test installed DXVK version
    let version = Dxvk::get_version(get_prefix_dir())?;

    assert!(version.is_some());
    assert_eq!(version.unwrap(), String::from("2.1"));

    wine.uninstall_dxvk(InstallParams::default())?;

    Ok(())
}

#[test]
#[serial]
fn create_prefix() {
    let wine = get_custom_wine();

    let result = wine.update_prefix(get_prefix_dir());

    assert!(result.is_ok());
    assert!(get_prefix_dir().join("drive_c/windows").exists());

    std::fs::remove_dir_all(get_prefix_dir().join("drive_c/windows/system32/drivers")).unwrap();

    let result = wine.update_prefix(get_prefix_dir());

    assert!(result.is_ok());
    assert!(get_prefix_dir().join("drive_c/windows/system32/drivers").exists());
}

#[test]
#[serial]
fn run_and_kill_notepad() {
    let wine = get_custom_wine().with_prefix(get_prefix_dir());

    assert!(wine.run("notepad").is_ok());

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Sometimes doesn't stop notepad process so I'm also ending wineboot session here
    assert!(wine.stop_processes(true).is_ok());
    assert!(wine.end_session().is_ok());
}
