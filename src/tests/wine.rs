use std::process::Command;

use serial_test::*;

use crate::prelude::*;
use super::*;

const CUSTOM_WINE: (&str, &str) = ("wine-9.22-staging-tkg-amd64", "https://github.com/Kron4ek/Wine-Builds/releases/download/9.22/wine-9.22-staging-tkg-amd64.tar.xz");

pub fn get_prefix_dir() -> PathBuf {
    get_test_dir().join("wine-prefix")
}

pub fn get_custom_wine() -> Wine {
    let test_dir = get_test_dir();

    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)
            .expect("Failed to create test directory");
    }

    let wine_dir = test_dir.join(CUSTOM_WINE.0);

    if !wine_dir.exists() {
        Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg(CUSTOM_WINE.1)
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

    Wine::from_binary(wine_dir.join("bin/wine"))
        .with_prefix(get_prefix_dir())
        .with_loader(WineLoader::Current)
        .with_arch(WineArch::Win64)
}

#[test]
#[parallel]
fn wine_version() -> anyhow::Result<()> {
    assert!(Wine::from_binary("\0").version().is_err());
    assert_eq!(get_custom_wine().version()?, "wine-9.22.r0.g7ba8823e ( TkG Staging Esync Fsync )\n");

    Ok(())
}

#[test]
#[serial]
fn create_prefix() -> anyhow::Result<()> {
    let wine = get_custom_wine();

    // Create wine prefix
    wine.init_prefix(None::<&str>)?;

    assert!(get_prefix_dir().join("drive_c/windows/system32/drivers").exists());

    // Remove drivers subfolder
    std::fs::remove_dir_all(get_prefix_dir().join("drive_c/windows/system32/drivers"))?;

    // Try to repair it
    wine.update_prefix(None::<&str>)?;

    assert!(get_prefix_dir().join("drive_c/windows/system32/drivers").exists());

    Ok(())
}

#[test]
#[serial]
fn run_and_kill_notepad() -> anyhow::Result<()> {
    let wine = get_custom_wine();

    let notepad = wine.run("notepad")?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Sometimes doesn't stop notepad process so I'm also ending wineboot session here
    wine.stop_processes(true)?;
    wine.end_session()?;

    assert!(notepad.wait_with_output()?.status.success());

    Ok(())
}
