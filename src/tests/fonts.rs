use serial_test::*;

use crate::wine::ext::{WineFontsExt, Corefont};
use super::wine::get_custom_wine;

#[test]
#[serial]
fn install_corefonts() -> anyhow::Result<()> {
    let wine = get_custom_wine();

    // for font in Corefont::iterator() {
    //     assert!(!wine.is_installed(font.to_str()));
    // }

    wine.install_corefonts()?;

    for font in Corefont::iterator() {
        assert!(wine.is_installed(font.to_str()));
    }

    Ok(())
}
