use std::process::Command;

use serial_test::*;

use crate::prelude::*;
use super::*;

const DXVK: (&str, &str) = ("dxvk-2.1", "https://github.com/doitsujin/dxvk/releases/download/v2.1/dxvk-2.1.tar.gz");

fn get_dxvk_folder() -> PathBuf {
    let test_dir = get_test_dir();

    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)
            .expect("Failed to create test directory");
    }

    let dxvk_dir = test_dir.join(DXVK.0);

    if !dxvk_dir.exists() {
        Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg(DXVK.1)
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
#[serial]
fn apply_dxvk() -> anyhow::Result<()> {
    // Test non existing prefix version
    assert!(Dxvk::get_version("\0").is_err());

    // Test DXVK uninstalling
    let dxvk_folder = get_dxvk_folder();
    let wine = wine::get_custom_wine().with_prefix(wine::get_prefix_dir());

    #[allow(unused_must_use)]
    {
        wine.uninstall_dxvk(InstallParams::default());
    }

    // Test clear prefix DXVK version
    assert_eq!(Dxvk::get_version(wine::get_prefix_dir())?, None);

    // Test DXVK installing
    wine.install_dxvk(dxvk_folder, InstallParams::default())?;

    // Test installed DXVK version
    assert_eq!(Dxvk::get_version(wine::get_prefix_dir())?, Some(String::from("2.1")));

    wine.uninstall_dxvk(InstallParams::default())?;

    // Test DXVK version after its uninstallation
    assert_eq!(Dxvk::get_version(wine::get_prefix_dir())?, None);

    Ok(())
}
