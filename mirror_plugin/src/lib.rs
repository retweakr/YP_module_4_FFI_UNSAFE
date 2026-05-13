use serde::Deserialize;
use std::ffi::{c_char, CStr};
use std::slice;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(default)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool,
}

#[no_mangle]
pub extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    if rgba_data.is_null() {
        return;
    }

    let Some(buffer_len) = buffer_len(width, height) else {
        return;
    };

    let parsed_params = parse_params(params);
    if !parsed_params.horizontal && !parsed_params.vertical {
        return;
    }

    let rgba_data = unsafe {
        // Безопасно, потому что основная программа передаёт валидный указатель на буфер
        // размером width * height * 4 байта, а переполнение мы уже проверили.
        slice::from_raw_parts_mut(rgba_data, buffer_len)
    };

    apply_mirror(rgba_data, width as usize, height as usize, parsed_params);
}

fn apply_mirror(buffer: &mut [u8], width: usize, height: usize, params: MirrorParams) {
    if params.horizontal {
        mirror_horizontally(buffer, width, height);
    }

    if params.vertical {
        mirror_vertically(buffer, width, height);
    }
}

fn mirror_horizontally(buffer: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..(width / 2) {
            let left = pixel_offset(x, y, width);
            let right = pixel_offset(width - 1 - x, y, width);
            swap_pixel(buffer, left, right);
        }
    }
}

fn mirror_vertically(buffer: &mut [u8], width: usize, height: usize) {
    for y in 0..(height / 2) {
        for x in 0..width {
            let top = pixel_offset(x, y, width);
            let bottom = pixel_offset(x, height - 1 - y, width);
            swap_pixel(buffer, top, bottom);
        }
    }
}

fn swap_pixel(buffer: &mut [u8], left: usize, right: usize) {
    for channel in 0..4 {
        buffer.swap(left + channel, right + channel);
    }
}

fn pixel_offset(x: usize, y: usize, width: usize) -> usize {
    (y * width + x) * 4
}

fn buffer_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
}

fn parse_params(params: *const c_char) -> MirrorParams {
    if params.is_null() {
        return MirrorParams::default();
    }

    let params = unsafe { CStr::from_ptr(params) };
    match params.to_str() {
        Ok(raw) if raw.trim().is_empty() => MirrorParams::default(),
        Ok(raw) => serde_json::from_str(raw).unwrap_or_default(),
        Err(_) => MirrorParams::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_mirror, MirrorParams};

    #[test]
    fn mirrors_horizontally() {
        let mut buffer = pixels([
            [1, 0, 0, 255],
            [2, 0, 0, 255],
            [3, 0, 0, 255],
            [4, 0, 0, 255],
        ]);

        apply_mirror(
            &mut buffer,
            2,
            2,
            MirrorParams {
                horizontal: true,
                vertical: false,
            },
        );

        assert_eq!(
            buffer,
            pixels([
                [2, 0, 0, 255],
                [1, 0, 0, 255],
                [4, 0, 0, 255],
                [3, 0, 0, 255],
            ])
        );
    }

    #[test]
    fn mirrors_vertically() {
        let mut buffer = pixels([
            [1, 0, 0, 255],
            [2, 0, 0, 255],
            [3, 0, 0, 255],
            [4, 0, 0, 255],
        ]);

        apply_mirror(
            &mut buffer,
            2,
            2,
            MirrorParams {
                horizontal: false,
                vertical: true,
            },
        );

        assert_eq!(
            buffer,
            pixels([
                [3, 0, 0, 255],
                [4, 0, 0, 255],
                [1, 0, 0, 255],
                [2, 0, 0, 255],
            ])
        );
    }

    #[test]
    fn mirrors_horizontally_and_vertically() {
        let mut buffer = pixels([
            [1, 0, 0, 255],
            [2, 0, 0, 255],
            [3, 0, 0, 255],
            [4, 0, 0, 255],
        ]);

        apply_mirror(
            &mut buffer,
            2,
            2,
            MirrorParams {
                horizontal: true,
                vertical: true,
            },
        );

        assert_eq!(
            buffer,
            pixels([
                [4, 0, 0, 255],
                [3, 0, 0, 255],
                [2, 0, 0, 255],
                [1, 0, 0, 255],
            ])
        );
    }

    fn pixels<const N: usize>(pixels: [[u8; 4]; N]) -> Vec<u8> {
        pixels.into_iter().flat_map(|pixel| pixel.into_iter()).collect()
    }
}
