pub mod error;
pub mod plugin_loader;

use clap::Parser;
use error::AppError;
use image::{ImageFormat, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "CLI image processor with FFI plugins",
    arg_required_else_help = true
)]
pub struct CliArgs {
    /// Путь к входному PNG-файлу
    pub input: PathBuf,

    /// Путь, куда будет сохранён результат
    pub output: PathBuf,

    /// Имя плагина без расширения файла
    pub plugin: String,

    /// Путь к файлу с параметрами плагина
    pub params: PathBuf,

    /// Папка, где лежат динамические библиотеки плагинов
    #[arg(long = "plugin-path", default_value = "target/debug")]
    pub plugin_path: PathBuf,
}

pub fn run(args: CliArgs) -> Result<(), AppError> {
    validate_existing_file(&args.input, "input image")?;
    validate_existing_file(&args.params, "params file")?;

    let params = fs::read_to_string(&args.params).map_err(|source| AppError::ReadParams {
        path: args.params.clone(),
        source,
    })?;

    let image = image::open(&args.input).map_err(|source| AppError::LoadImage {
        path: args.input.clone(),
        source,
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut raw_pixels = rgba.into_raw();

    plugin_loader::apply_plugin(
        &args.plugin_path,
        &args.plugin,
        width,
        height,
        &mut raw_pixels,
        &params,
    )?;

    let output_image =
        RgbaImage::from_raw(width, height, raw_pixels).ok_or(AppError::RebuildImageBuffer)?;
    output_image
        .save_with_format(&args.output, ImageFormat::Png)
        .map_err(|source| AppError::SaveImage {
            path: args.output.clone(),
            source,
        })?;

    Ok(())
}

fn validate_existing_file(path: &Path, label: &'static str) -> Result<(), AppError> {
    if !path.exists() {
        return Err(AppError::MissingPath {
            label,
            path: path.to_path_buf(),
        });
    }

    if !path.is_file() {
        return Err(AppError::PathIsNotFile {
            label,
            path: path.to_path_buf(),
        });
    }

    Ok(())
}
