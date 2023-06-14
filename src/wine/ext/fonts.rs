use std::process::{Command, Stdio};

use crate::wine::*;
use crate::wine::ext::WineRunExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    /// Source: https://mirrors.kernel.org/gentoo/distfiles/andale32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | AndaleMo.TTF | andalemo.ttf | Andale Mono |
    Andale,

    /// Source (1): https://mirrors.kernel.org/gentoo/distfiles/arial32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Arial.TTF | arial.ttf | Arial |
    /// | Arialbd.TTF | arialbd.ttf | Arial Bold |
    /// | Ariali.TTF | ariali.ttf | Arial Italic |
    /// | Arialbi.TTF | arialbi.ttf | Arial Bold Italic |
    /// 
    /// Source (2): https://mirrors.kernel.org/gentoo/distfiles/arialb32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | AriBlk.TTF | ariblk.ttf | Arial Black |
    Arial,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/comic32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Comic.TTF | comic.ttf | Comic Sans MS |
    /// | Comicbd.TTF | comicbd.ttf | Comic Sans MS Bold |
    ComicSans,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/courie32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | cour.ttf | cour.ttf | Courier New |
    /// | courbd.ttf | courbd.ttf | Courier New Bold |
    /// | couri.ttf | couri.ttf | Courier New Italic |
    /// | courbi.ttf | courbi.ttf | Courier New Bold Italic |
    Courier,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/georgi32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Georgia.TTF | georgia.ttf | Georgia |
    /// | Georgiab.TTF | georgiab.ttf | Georgia Bold |
    /// | Georgiai.TTF | georgiai.ttf | Georgia Italic |
    /// | Georgiaz.TTF | georgiaz.ttf | Georgia Bold Italic |
    Georgia,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/impact32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Impact.TTF | impact.ttf | Impact |
    Impact,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/times32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Times.TTF | times.ttf | Times New Roman |
    /// | Timesbd.TTF | timesbd.ttf | Times New Roman Bold |
    /// | Timesi.TTF | timesi.ttf | Times New Roman Italic |
    /// | Timesbi.TTF | timesbi.ttf | Times New Roman Bold Italic |
    Times,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/trebuc32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | trebuc.ttf | trebuc.ttf | Trebuchet MS |
    /// | Trebucbd.ttf | trebucbd.ttf | Trebuchet MS Bold |
    /// | trebucit.ttf | trebucit.ttf | Trebuchet MS Italic |
    /// | trebucbi.ttf | trebucbi.ttf | Trebuchet MS Bold Italic |
    Trebuchet,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/verdan32.exe
    /// 
    /// | File | Winetricks File | Name |
    /// | :- | :- | :- |
    /// | Verdana.TTF | verdana.ttf | Verdana |
    /// | Verdanab.TTF | verdanab.ttf | Verdana Bold |
    /// | Verdanai.TTF | verdanai.ttf | Verdana Italic |
    /// | Verdanaz.TTF | verdanaz.ttf | Verdana Bold Italic |
    Verdana,

    /// Source: https://mirrors.kernel.org/gentoo/distfiles/webdin32.exe
    /// 
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

// TODO: is_installed method name can be kind of weird, if people import prelude they might have no idea what it means

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

    /// Install given Microsoft Corefont
    /// 
    /// ```no_run
    /// use wincompatlib::wine::Wine;
    /// use wincompatlib::wine::ext::{WineFontsExt, Corefont};
    /// 
    /// if let Err(err) = Wine::default().install_corefont(Corefont::Times) {
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
        fn install_fonts(wine: &Wine, url: &str, install: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>, impl AsRef<str>)>) -> anyhow::Result<()> {
            let name = url.split('/').last().unwrap().strip_suffix(".exe").unwrap();

            // FIXME: folder name can be lowercased?
            let fonts = wine.prefix.join("drive_c/windows/Fonts");
            let cabextract_temp = fonts.join(format!(".{name}-cabextract"));

            if cabextract_temp.exists() {
                std::fs::remove_dir_all(&cabextract_temp)?;
            }

            std::fs::create_dir(&cabextract_temp)?;

            let path = cabextract_temp.join(format!("{name}.exe"));
            let temp = cabextract_temp.join(name);

            std::fs::write(&path, minreq::get(url).send()?.as_bytes())?;

            let output = Command::new("cabextract")
                .arg("-d")
                .arg(&temp)
                .arg(path)
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

            Ok(())
        }

        match font {
            Font::Andale => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/andale32.exe", [
                ("AndaleMo.TTF", "andalemo.ttf", "Andale Mono")
            ])?,

            Font::Arial => {
                install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/arial32.exe", [
                    ("Arial.TTF",   "arial.ttf",   "Arial"),
                    ("Arialbd.TTF", "arialbd.ttf", "Arial Bold"),
                    ("Ariali.TTF",  "ariali.ttf",  "Arial Italic"),
                    ("Arialbi.TTF", "arialbi.ttf", "Arial Bold Italic")
                ])?;

                install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/arialb32.exe", [
                    ("AriBlk.TTF", "ariblk.ttf", "Arial Black")
                ])?;
            }

            Font::ComicSans => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/comic32.exe", [
                ("Comic.TTF",   "comic.ttf",   "Comic Sans MS"),
                ("Comicbd.TTF", "comicbd.ttf", "Comic Sans MS Bold"),
            ])?,

            Font::Courier => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/courie32.exe", [
                ("cour.ttf",   "cour.ttf",   "Courier New"),
                ("courbd.ttf", "courbd.ttf", "Courier New Bold"),
                ("couri.ttf",  "couri.ttf",  "Courier New Italic"),
                ("courbi.ttf", "courbi.ttf", "Courier New Bold Italic")
            ])?,

            Font::Georgia => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/georgi32.exe", [
                ("Georgia.TTF",  "georgia.ttf",  "Georgia"),
                ("Georgiab.TTF", "georgiab.ttf", "Georgia Bold"),
                ("Georgiai.TTF", "georgiai.ttf", "Georgia Italic"),
                ("Georgiaz.TTF", "georgiaz.ttf", "Georgia Bold Italic")
            ])?,

            Font::Impact => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/impact32.exe", [
                ("Impact.TTF", "impact.ttf", "Impact")
            ])?,

            Font::Times => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/times32.exe", [
                ("Times.TTF",   "times.ttf",   "Times New Roman"),
                ("Timesbd.TTF", "timesbd.ttf", "Times New Roman Bold"),
                ("Timesi.TTF",  "timesi.ttf",  "Times New Roman Italic"),
                ("Timesbi.TTF", "timesbi.ttf", "Times New Roman Bold Italic")
            ])?,

            Font::Trebuchet => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/trebuc32.exe", [
                ("trebuc.ttf",   "trebuc.ttf",   "Trebuchet MS"),
                ("Trebucbd.ttf", "trebucbd.ttf", "Trebuchet MS Bold"),
                ("trebucit.ttf", "trebucit.ttf", "Trebuchet MS Italic"),
                ("trebucbi.ttf", "trebucbi.ttf", "Trebuchet MS Bold Italic")
            ])?,

            Font::Verdana => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/verdan32.exe", [
                ("Verdana.TTF",  "verdana.ttf",  "Verdana"),
                ("Verdanab.TTF", "verdanab.ttf", "Verdana Bold"),
                ("Verdanai.TTF", "verdanai.ttf", "Verdana Italic"),
                ("Verdanaz.TTF", "verdanaz.ttf", "Verdana Bold Italic")
            ])?,

            Font::Webdings => install_fonts(self, "https://mirrors.kernel.org/gentoo/distfiles/webdin32.exe", [
                ("Webdings.TTF", "webdings.ttf", "Webdings")
            ])?,
        }

        Ok(())
    }
}
