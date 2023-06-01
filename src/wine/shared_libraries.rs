use std::path::PathBuf;

const WINE_LIBS: &[&str] = &[
    "lib",
    "lib64",
    "lib/wine/x86_64-unix",
    "lib32/wine/x86_64-unix",
    "lib64/wine/x86_64-unix",
    "lib/wine/i386-unix",
    "lib32/wine/i386-unix",
    "lib64/wine/i386-unix"
];

const GSTREAMER_LIBS: &[&str] = &[
    "lib64/gstreamer-1.0",
    "lib/gstreamer-1.0",
    "lib32/gstreamer-1.0"
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wine {
    /// Don't set `LD_LIBRARY_PATH` variable
    None,

    /// Set `LD_LIBRARY_PATH` with standard wine libraries paths in the given wine build folder
    Standard(PathBuf),

    /// Set `LD_LIBRARY_PATH` with custom libraries paths
    Custom(Vec<PathBuf>)
}

impl Default for Wine {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl Wine {
    pub fn get_paths(&self) -> Option<String> {
        match self {
            Self::None => None,

            Self::Standard(path) => Some(WINE_LIBS.iter()
                .map(|folder| path.join(folder))
                .fold(String::new(), |paths, path| format!("{paths}:{}", path.to_string_lossy()))),

            Self::Custom(paths) => Some(paths.iter()
                .fold(String::new(), |paths, path| format!("{paths}:{}", path.to_string_lossy())))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gstreamer {
    /// Don't set `GST_PLUGIN_PATH` variable
    None,

    /// Set `GST_PLUGIN_PATH` with standard gstreamer libraries paths in the given wine build folder
    Standard(PathBuf),

    /// Set `GST_PLUGIN_PATH` with custom libraries paths
    Custom(Vec<PathBuf>)
}

impl Default for Gstreamer {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl Gstreamer {
    pub fn get_paths(&self) -> Option<String> {
        match self {
            Self::None => None,

            Self::Standard(path) => Some(GSTREAMER_LIBS.iter()
                .map(|folder| path.join(folder))
                .fold(String::new(), |paths, path| format!("{paths}:{}", path.to_string_lossy()))),

            Self::Custom(paths) => Some(paths.iter()
                .fold(String::new(), |paths, path| format!("{paths}:{}", path.to_string_lossy())))
        }
    }
}
