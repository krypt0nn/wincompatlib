use std::process::Command;
use std::ffi::OsString;
use std::str::FromStr;

use serial_test::*;

use crate::prelude::*;
use super::*;

#[cfg(feature = "wine-proton")]
const CUSTOM_PROTON: (&str, &str) = ("GE-Proton7-50", "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/GE-Proton7-50/GE-Proton7-50.tar.gz");

pub fn get_prefix_dir() -> PathBuf {
    get_test_dir().join("proton-prefix")
}

#[cfg(feature = "wine-proton")]
fn get_custom_proton() -> Proton {
    let test_dir = get_test_dir();

    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)
            .expect("Failed to create test directory");
    }

    let proton_dir = test_dir.join(CUSTOM_PROTON.0);

    if !proton_dir.exists() {
        Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg(CUSTOM_PROTON.1)
            .arg("-o")
            .arg(test_dir.join("proton.tar.gz"))
            .output()
            .expect("Failed to download proton. Curl is not available?");

        Command::new("tar")
            .arg("-xf")
            .arg("proton.tar.gz")
            .current_dir(test_dir)
            .output()
            .expect("Failed to extract downloaded proton. Tar is not available?");
    }

    Proton::new(proton_dir, None)
        .with_prefix(get_prefix_dir())
}

#[test]
#[parallel]
fn proton_version() -> std::io::Result<()> {
    assert_eq!(get_custom_proton().wine().version()?, "wine-7.0 (Staging)\n");

    Ok(())
}

#[test]
#[serial]
fn create_prefix() -> std::io::Result<()> {
    let proton = get_custom_proton();
    let wine_prefix = proton.wine().to_owned().prefix.unwrap();

    // Create wine prefix
    proton.update_prefix(&wine_prefix)?;

    assert!(wine_prefix.join("drive_c/windows/system32/drivers").exists());

    // Remove drivers subfolder
    std::fs::remove_dir_all(wine_prefix.join("drive_c/windows/system32/drivers"))?;

    // Try to repair it
    proton.update_prefix(&wine_prefix)?;

    assert!(wine_prefix.join("drive_c/windows/system32/drivers").exists());

    Ok(())
}

#[test]
#[serial]
fn run_and_kill_notepad() -> std::io::Result<()> {
    let proton = get_custom_proton();

    let notepad = proton.run("notepad")?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Sometimes doesn't stop notepad process so I'm also ending wineboot session here
    proton.stop_processes(true)?;
    proton.end_session()?;

    dbg!(notepad.wait_with_output());

    // assert!(notepad.wait_with_output()?.status.success());

    Ok(())
}
