use std::process::Command;

use serial_test::*;

use crate::prelude::*;
use super::*;

const CUSTOM_PROTON: (&str, &str) = ("GE-Proton9-27", "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/GE-Proton9-27/GE-Proton9-27.tar.gz");

pub fn get_prefix_dir() -> PathBuf {
    get_test_dir().join("proton-prefix")
}

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
fn proton_version() -> anyhow::Result<()> {
    assert_eq!(get_custom_proton().wine().version()?, "wine-9.0 (Staging)\n");

    Ok(())
}

#[test]
#[serial]
fn create_prefix() -> anyhow::Result<()> {
    let proton = get_custom_proton();
    let wine_prefix = proton.wine().to_owned().prefix;

    // Create wine prefix
    proton.update_prefix(None::<&str>)?;

    assert!(wine_prefix.join("drive_c/windows/system32/drivers").exists());

    // Remove drivers subfolder
    std::fs::remove_dir_all(wine_prefix.join("drive_c/windows/system32/drivers"))?;

    // Try to repair it
    proton.update_prefix(None::<&str>)?;

    assert!(wine_prefix.join("drive_c/windows/system32/drivers").exists());

    Ok(())
}

#[test]
#[serial]
fn run_and_kill_notepad() -> anyhow::Result<()> {
    let proton = get_custom_proton();

    // Never works well so..
    // let notepad = proton.run_in_prefix("notepad")?;

    let notepad = proton.wine().run("notepad")?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Sometimes doesn't stop notepad process so I'm also ending wineboot session here
    proton.stop_processes(true)?;
    proton.end_session()?;

    assert!(notepad.wait_with_output()?.status.success());

    Ok(())
}
