use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing {label}: {path}", path = .path.display())]
    MissingPath { label: &'static str, path: PathBuf },

    #[error("{label} is not a file: {path}", path = .path.display())]
    PathIsNotFile { label: &'static str, path: PathBuf },

    #[error("Failed to read params file {path}: {source}", path = .path.display())]
    ReadParams {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to open image {path}: {source}", path = .path.display())]
    LoadImage {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("Failed to save image {path}: {source}", path = .path.display())]
    SaveImage {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("Plugin library not found: {path}", path = .path.display())]
    MissingPluginLibrary { path: PathBuf },

    #[error("Failed to load plugin library {path}: {source}", path = .path.display())]
    LoadPluginLibrary {
        path: PathBuf,
        #[source]
        source: libloading::Error,
    },

    #[error("Failed to find process_image in {path}: {source}", path = .path.display())]
    LoadPluginSymbol {
        path: PathBuf,
        #[source]
        source: libloading::Error,
    },

    #[error("RGBA buffer size overflow for image dimensions {width}x{height}")]
    BufferSizeOverflow { width: u32, height: u32 },

    #[error("Invalid RGBA buffer length: expected {expected} bytes, got {actual}")]
    InvalidBufferLength { expected: usize, actual: usize },

    #[error("Plugin params contain an interior NUL byte")]
    InvalidParamsString(#[from] std::ffi::NulError),

    #[error("Failed to rebuild RGBA image buffer after plugin processing")]
    RebuildImageBuffer,
}
