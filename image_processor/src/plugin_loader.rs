use crate::error::AppError;
use libloading::{Library, Symbol};
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::ffi::{c_char, CString};
use std::path::{Path, PathBuf};

type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

pub fn apply_plugin(
    plugin_dir: &Path,
    plugin_name: &str,
    width: u32,
    height: u32,
    rgba_data: &mut [u8],
    params: &str,
) -> Result<(), AppError> {
    let expected_len = expected_buffer_len(width, height)?;
    if rgba_data.len() != expected_len {
        return Err(AppError::InvalidBufferLength {
            expected: expected_len,
            actual: rgba_data.len(),
        });
    }

    let library_path = plugin_library_path(plugin_dir, plugin_name);
    if !library_path.is_file() {
        return Err(AppError::MissingPluginLibrary { path: library_path });
    }

    let params = CString::new(params)?;
    let library = unsafe {
        Library::new(&library_path).map_err(|source| AppError::LoadPluginLibrary {
            path: library_path.clone(),
            source,
        })?
    };

    let process_image: Symbol<'_, ProcessImageFn> = unsafe {
        library
            .get(b"process_image\0")
            .map_err(|source| AppError::LoadPluginSymbol {
                path: library_path.clone(),
                source,
            })?
    };

    unsafe {
        // Безопасно, потому что длина буфера точно равна width * height * 4,
        // буфер живёт до конца вызова, а CString даёт корректную C-строку.
        process_image(width, height, rgba_data.as_mut_ptr(), params.as_ptr());
    }

    Ok(())
}

fn plugin_library_path(plugin_dir: &Path, plugin_name: &str) -> PathBuf {
    plugin_dir.join(format!("{DLL_PREFIX}{plugin_name}{DLL_SUFFIX}"))
}

fn expected_buffer_len(width: u32, height: u32) -> Result<usize, AppError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(AppError::BufferSizeOverflow { width, height })
}
