use serial_test::*;

use crate::wine::ext::{WineFontsExt, Font};
use super::wine::get_custom_wine;

#[test]
#[serial]
fn install_all_fonts() -> anyhow::Result<()> {
    let wine = get_custom_wine();

    for font in Font::iterator() {
        if !font.is_installed(&wine.prefix) {
            wine.install_font(font)?;

            assert!(font.is_installed(&wine.prefix));
        }
    }

    Ok(())
}
