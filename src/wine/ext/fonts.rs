use std::process::{Command, Stdio};

use crate::wine::*;
use crate::wine::ext::WineRunExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | AndaleMo.TTF | andalemo.ttf | Andale Mono |
    Andale,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Arial.TTF | arial.ttf | Arial |
    /// | Arialbd.TTF | arialbd.ttf | Arial Bold |
    /// | Ariali.TTF | ariali.ttf | Arial Italic |
    /// | Arialbi.TTF | arialbi.ttf | Arial Bold Italic |
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | AriBlk.TTF | ariblk.ttf | Arial Black |
    Arial,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Comic.TTF | comic.ttf | Comic Sans MS |
    /// | Comicbd.TTF | comicbd.ttf | Comic Sans MS Bold |
    ComicSans,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | cour.ttf | cour.ttf | Courier New |
    /// | courbd.ttf | courbd.ttf | Courier New Bold |
    /// | couri.ttf | couri.ttf | Courier New Italic |
    /// | courbi.ttf | courbi.ttf | Courier New Bold Italic |
    Courier,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Georgia.TTF | georgia.ttf | Georgia |
    /// | Georgiab.TTF | georgiab.ttf | Georgia Bold |
    /// | Georgiai.TTF | georgiai.ttf | Georgia Italic |
    /// | Georgiaz.TTF | georgiaz.ttf | Georgia Bold Italic |
    Georgia,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Impact.TTF | impact.ttf | Impact |
    Impact,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Times.TTF | times.ttf | Times New Roman |
    /// | Timesbd.TTF | timesbd.ttf | Times New Roman Bold |
    /// | Timesi.TTF | timesi.ttf | Times New Roman Italic |
    /// | Timesbi.TTF | timesbi.ttf | Times New Roman Bold Italic |
    Times,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | trebuc.ttf | trebuc.ttf | Trebuchet MS |
    /// | Trebucbd.ttf | trebucbd.ttf | Trebuchet MS Bold |
    /// | trebucit.ttf | trebucit.ttf | Trebuchet MS Italic |
    /// | trebucbi.ttf | trebucbi.ttf | Trebuchet MS Bold Italic |
    Trebuchet,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Verdana.TTF | verdana.ttf | Verdana |
    /// | Verdanab.TTF | verdanab.ttf | Verdana Bold |
    /// | Verdanai.TTF | verdanai.ttf | Verdana Italic |
    /// | Verdanaz.TTF | verdanaz.ttf | Verdana Bold Italic |
    Verdana,

    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Webdings.TTF | webdings.ttf | Webdings |
    Webdings
}

impl Font {
    /// Get iterator over all available enum values
    pub fn iterator() -> impl IntoIterator<Item = Self> {
        [
            Self::Andale,
            Self::Arial,
            Self::ComicSans,
            Self::Courier,
            Self::Georgia,
            Self::Impact,
            Self::Times,
            Self::Trebuchet,
            Self::Verdana,
            Self::Webdings
        ].into_iter()
    }

    /// Get corefont code name
    /// 
    /// | Corefont enum | Font code name |
    /// | :- | :- |
    /// | Andale | andalemo |
    /// | Arial | arial |
    /// | ComicSans | comic |
    /// | Courier | cour |
    /// | Georgia | georgia |
    /// | Impact | impact |
    /// | Times | times |
    /// | Trebuchet | trebuc |
    /// | Verdana | verdana |
    /// | Webdings | webdings |
    pub fn code(&'_ self) -> &'_ str {
        match self {
            Self::Andale    => "andalemo",
            Self::Arial     => "arial",
            Self::ComicSans => "comic",
            Self::Courier   => "cour",
            Self::Georgia   => "georgia",
            Self::Impact    => "impact",
            Self::Times     => "times",
            Self::Trebuchet => "trebuc",
            Self::Verdana   => "verdana",
            Self::Webdings  => "webdings"
        }
    }

    /// Get full corefont name
    /// 
    /// | Corefont enum | Font name |
    /// | :- | :- |
    /// | Andale | Andale |
    /// | Arial | Arial |
    /// | ComicSans | Comic Sans MS |
    /// | Courier | Courier New |
    /// | Georgia | Georgia |
    /// | Impact | Impact |
    /// | Times | Times New Roman |
    /// | Trebuchet | Trebuchet MS |
    /// | Verdana | Verdana |
    /// | Webdings | Webdings |
    pub fn name(&'_ self) -> &'_ str {
        match self {
            Self::Andale    => "Andale",
            Self::Arial     => "Arial",
            Self::ComicSans => "Comic Sans MS",
            Self::Courier   => "Courier New",
            Self::Georgia   => "Georgia",
            Self::Impact    => "Impact",
            Self::Times     => "Times New Roman",
            Self::Trebuchet => "Trebuchet MS",
            Self::Verdana   => "Verdana",
            Self::Webdings  => "Webdings"
        }
    }

    /// Check if current font is installed in the wine prefix's fonts folder
    pub fn is_installed(&self, prefix: impl AsRef<Path>) -> bool {
        let prefix = prefix.as_ref();
        let font = self.code();

        prefix.join("drive_c/windows/Fonts").join(format!("{font}.ttf")).exists() |
        prefix.join("drive_c/windows/Fonts").join(format!("{font}.TTF")).exists() |

        // Didn't see such situations in real life but it's listed in the winetricks so I guess they can occur?
        prefix.join("drive_c/windows/fonts").join(format!("{font}.ttf")).exists() |
        prefix.join("drive_c/windows/fonts").join(format!("{font}.TTF")).exists()
    }
}

pub trait WineFontsExt {
    /// Register font in the wine registry
    /// 
    /// ```no_run
    /// use wincompatlib::wine::Wine;
    /// use wincompatlib::wine::ext::WineFontsExt;
    /// 
    /// // times.ttf should be in the wine fonts directory
    /// if let Err(err) = Wine::default().register_font("times.ttf", "Times New Roman") {
    ///     eprintln!("Failed to register Times New Roman font: {err}");
    /// }
    /// ```
    fn register_font(&self, ttf: impl AsRef<str>, font_name: impl AsRef<str>) -> anyhow::Result<()>;

    /// Check if ttf with given name is installed in the wine fonts folder
    /// 
    /// ```
    /// use wincompatlib::wine::Wine;
    /// use wincompatlib::wine::ext::WineFontsExt;
    /// 
    /// let installed = Wine::default().font_is_installed("times");
    /// 
    /// println!("Is Times fonts installed: {:?}", installed);
    /// ```
    fn font_is_installed(&self, ttf: impl AsRef<str>) -> bool;

    /// Install given font
    /// 
    /// ```no_run
    /// use wincompatlib::wine::Wine;
    /// use wincompatlib::wine::ext::{WineFontsExt, Font};
    /// 
    /// if let Err(err) = Wine::default().install_font(Font::Times) {
    ///     eprintln!("Failed to install Times New Roman: {err}");
    /// }
    /// ```
    fn install_font(&self, font: Font) -> anyhow::Result<()>;
}

impl WineFontsExt for Wine {
    fn register_font(&self, font_file: impl AsRef<str>, font_name: impl AsRef<str>) -> anyhow::Result<()> {
        // "$wine" reg add HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts /f font.ttf /d "Font Name" /f
        let output = self.run_args(["reg", "add", "HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts", "/v", font_name.as_ref(), "/d", font_file.as_ref(), "/f"])?
            .wait_with_output()?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error = stdout.trim_end().lines().last().unwrap_or(&stdout);

            anyhow::bail!("Failed to register font: {error}");
        }

        // HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows\\CurrentVersion\\Fonts
        let output = self.run_args(["reg", "add", "HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows\\CurrentVersion\\Fonts", "/v", font_name.as_ref(), "/d", font_file.as_ref(), "/f"])?
            .wait_with_output()?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error = stdout.trim_end().lines().last().unwrap_or(&stdout);

            anyhow::bail!("Failed to register font: {error}");
        }

        Ok(())
    }

    fn font_is_installed(&self, font_file: impl AsRef<str>) -> bool {
        self.prefix.join("drive_c/windows/Fonts").join(font_file.as_ref()).exists() |
        self.prefix.join("drive_c/windows/Fonts").join(format!("{}.ttf", font_file.as_ref())).exists() |
        self.prefix.join("drive_c/windows/Fonts").join(format!("{}.TTF", font_file.as_ref())).exists() |

        // Didn't see such situations in real life but it's listed in the winetricks so I guess they can occur?
        self.prefix.join("drive_c/windows/fonts").join(font_file.as_ref()).exists() |
        self.prefix.join("drive_c/windows/fonts").join(format!("{}.ttf", font_file.as_ref())).exists() |
        self.prefix.join("drive_c/windows/fonts").join(format!("{}.TTF", font_file.as_ref())).exists()
    }

    // TODO: I've made a merge request to minreq to add is_ok method. Use it once it will be merged

    fn install_font(&self, font: Font) -> anyhow::Result<()> {
        fn install_fonts(wine: &Wine, font_name: &str, install: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>, impl AsRef<str>)>) -> anyhow::Result<()> {
            // Took them from https://salsa.debian.org/debian/msttcorefonts/-/blob/master/update-ms-fonts + added one mine
            const CDN_BASE_URLS: &[&str] = &[
                "https://downloads.sourceforge.net/corefonts",
                "https://jaist.dl.sourceforge.net/sourceforge/corefonts",
                "https://nchc.dl.sourceforge.net/sourceforge/corefonts",
                "https://ufpr.dl.sourceforge.net/sourceforge/corefonts",
                "https://internode.dl.sourceforge.net/sourceforge/corefonts",
                "https://netcologne.dl.sourceforge.net/sourceforge/corefonts",
                "https://vorboss.dl.sourceforge.net/sourceforge/corefonts",
                "https://netix.dl.sourceforge.net/sourceforge/corefonts"
            ];

            // Fonts blake3 hashes to verify their correctness
            const FONTS_HASHES: &[(&str, &str)] = &[
                ("andale32", "f794d32548caba2a2a2efd9625f9e268866445ddc3aea4a1353be86c529018fb"),
                ("arial32",  "3e1018c47291d18d94281dc94e2b36d1572dc28a08715507e1f05e1b710eccc7"),
                ("arialb32", "2b6f2332b61da519c535a3074f0ac1c76427c1db458833ab4ab20bd30c325296"),
                ("comic32",  "5df2f0d4f3a2af489b3cb6213ef4d1ff1dffe67d1842953a448ee0a1ce875896"),
                ("courie32", "6a1287b2e574cce551528d55457269d18f7930c8d4cf694caaea9f56913cc554"),
                ("georgi32", "2c53bcfa1bb77b4679e309db1261d08e0c896a7374b282f8b9a8080d1f05f54b"),
                ("impact32", "fe450901803f732a21d1d1b8081c62d7dfba1eba9b4a9501d56996b1e664681b"),
                ("times32",  "d1bb288a928748d31770eb70af0d0073cb0efeccde6108420a39d044c25d9006"),
                ("trebuc32", "7c5f5e3e6904f01803d0798f295b2a8152aa54912ca31f8ea675028a0dca71a1"),
                ("verdan32", "01f8aa9820d516b5e6109a215369726a9e4abbceb2bd77f77fbfad9d047a9994"),
                ("webdin32", "fe885f86c98d2bf96251088804e07e6e1164d0b9b05deedf12ea72aff6f6e894")
            ];

            // FIXME: folder name can be lowercased?
            let fonts = wine.prefix.join("drive_c/windows/Fonts");
            let cabextract_temp = fonts.join(format!(".{font_name}-cabextract"));

            if cabextract_temp.exists() {
                std::fs::remove_dir_all(&cabextract_temp)?;
            }

            std::fs::create_dir(&cabextract_temp)?;

            let path = cabextract_temp.join(format!("{font_name}.exe"));
            let temp = cabextract_temp.join(font_name);

            for url in CDN_BASE_URLS {
                if let Ok(content) = minreq::get(format!("{url}/{font_name}.exe")).send() {
                    let content = content.as_bytes();
                    let hash = blake3::hash(content).to_string();

                    for (font, font_hash) in FONTS_HASHES {
                        if font == &font_name && font_hash != &hash {
                            anyhow::bail!("Font {font_name} was downloaded from the CDN, but its hash is incorrect");
                        }
                    }

                    std::fs::write(&path, content)?;

                    let output = Command::new("cabextract")
                        .arg("-d")
                        .arg(&temp)
                        .arg(&path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?
                        .wait_with_output()?;

                    if !output.status.success() {
                        anyhow::bail!("Failed to cabextract font: {}", String::from_utf8_lossy(&output.stderr));
                    }

                    for (original, new, name) in install {
                        std::fs::copy(temp.join(original.as_ref()), fonts.join(new.as_ref()))?;

                        wine.register_font(new, name)?;
                    }

                    std::fs::remove_dir_all(cabextract_temp)?;

                    return Ok(());
                }
            }

            anyhow::bail!("Couldn't connect to any of the CDNs to download the {font_name} font");
        }

        match font {
            Font::Andale => install_fonts(self, "andale32", [
                ("AndaleMo.TTF", "andalemo.ttf", "Andale Mono")
            ])?,

            Font::Arial => {
                install_fonts(self, "arial32", [
                    ("Arial.TTF",   "arial.ttf",   "Arial"),
                    ("Arialbd.TTF", "arialbd.ttf", "Arial Bold"),
                    ("Ariali.TTF",  "ariali.ttf",  "Arial Italic"),
                    ("Arialbi.TTF", "arialbi.ttf", "Arial Bold Italic")
                ])?;

                install_fonts(self, "arialb32", [
                    ("AriBlk.TTF", "ariblk.ttf", "Arial Black")
                ])?;
            }

            Font::ComicSans => install_fonts(self, "comic32", [
                ("Comic.TTF",   "comic.ttf",   "Comic Sans MS"),
                ("Comicbd.TTF", "comicbd.ttf", "Comic Sans MS Bold"),
            ])?,

            Font::Courier => install_fonts(self, "courie32", [
                ("cour.ttf",   "cour.ttf",   "Courier New"),
                ("courbd.ttf", "courbd.ttf", "Courier New Bold"),
                ("couri.ttf",  "couri.ttf",  "Courier New Italic"),
                ("courbi.ttf", "courbi.ttf", "Courier New Bold Italic")
            ])?,

            Font::Georgia => install_fonts(self, "georgi32", [
                ("Georgia.TTF",  "georgia.ttf",  "Georgia"),
                ("Georgiab.TTF", "georgiab.ttf", "Georgia Bold"),
                ("Georgiai.TTF", "georgiai.ttf", "Georgia Italic"),
                ("Georgiaz.TTF", "georgiaz.ttf", "Georgia Bold Italic")
            ])?,

            Font::Impact => install_fonts(self, "impact32", [
                ("Impact.TTF", "impact.ttf", "Impact")
            ])?,

            Font::Times => install_fonts(self, "times32", [
                ("Times.TTF",   "times.ttf",   "Times New Roman"),
                ("Timesbd.TTF", "timesbd.ttf", "Times New Roman Bold"),
                ("Timesi.TTF",  "timesi.ttf",  "Times New Roman Italic"),
                ("Timesbi.TTF", "timesbi.ttf", "Times New Roman Bold Italic")
            ])?,

            Font::Trebuchet => install_fonts(self, "trebuc32", [
                ("trebuc.ttf",   "trebuc.ttf",   "Trebuchet MS"),
                ("Trebucbd.ttf", "trebucbd.ttf", "Trebuchet MS Bold"),
                ("trebucit.ttf", "trebucit.ttf", "Trebuchet MS Italic"),
                ("trebucbi.ttf", "trebucbi.ttf", "Trebuchet MS Bold Italic")
            ])?,

            Font::Verdana => install_fonts(self, "verdan32", [
                ("Verdana.TTF",  "verdana.ttf",  "Verdana"),
                ("Verdanab.TTF", "verdanab.ttf", "Verdana Bold"),
                ("Verdanai.TTF", "verdanai.ttf", "Verdana Italic"),
                ("Verdanaz.TTF", "verdanaz.ttf", "Verdana Bold Italic")
            ])?,

            Font::Webdings => install_fonts(self, "webdin32", [
                ("Webdings.TTF", "webdings.ttf", "Webdings")
            ])?,
        }

        Ok(())
    }
}
