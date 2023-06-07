use serial_test::*;

use crate::wine::ext::{WineFontsExt, Corefont};
use super::wine::get_custom_wine;

#[test]
#[serial]
fn install_corefonts() -> anyhow::Result<()> {
    let wine = get_custom_wine();

    for font in Corefont::iterator() {
        if !font.is_installed(&wine.prefix) {
            wine.install_corefont(font)?;

            assert!(font.is_installed(&wine.prefix));
        }
    }

    Ok(())
}
