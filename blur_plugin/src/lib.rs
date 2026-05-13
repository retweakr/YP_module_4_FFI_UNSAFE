use serde::Deserialize;
use std::ffi::{c_char, CStr};
use std::slice;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(default)]
struct BlurParams {
    radius: u32,
    iterations: u32,
}

impl Default for BlurParams {
    fn default() -> Self {
        Self {
            radius: 0,
            iterations: 1,
        }
    }
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
    let rgba_data = unsafe {
        // Безопасно, потому что основная программа передаёт валидный указатель на буфер
        // размером width * height * 4 байта, а переполнение мы уже проверили.
        slice::from_raw_parts_mut(rgba_data, buffer_len)
    };

    apply_blur(rgba_data, width as usize, height as usize, parsed_params);
}

fn apply_blur(buffer: &mut [u8], width: usize, height: usize, params: BlurParams) {
    if width == 0 || height == 0 || params.radius == 0 || params.iterations == 0 {
        return;
    }

    let radius = params.radius as usize;
    let mut working = buffer.to_vec();
    let mut output = working.clone();

    for _ in 0..params.iterations {
        blur_once(&working, &mut output, width, height, radius);
        std::mem::swap(&mut working, &mut output);
    }

    buffer.copy_from_slice(&working);
}

fn blur_once(source: &[u8], destination: &mut [u8], width: usize, height: usize, radius: usize) {
    let radius_sq = (radius as u64) * (radius as u64);

    for y in 0..height {
        for x in 0..width {
            let x_start = x.saturating_sub(radius);
            let x_end = (x + radius).min(width - 1);
            let y_start = y.saturating_sub(radius);
            let y_end = (y + radius).min(height - 1);

            let mut totals = [0.0_f32; 4];
            let mut total_weight = 0.0_f32;

            for ny in y_start..=y_end {
                for nx in x_start..=x_end {
                    let dx = nx.abs_diff(x) as u64;
                    let dy = ny.abs_diff(y) as u64;
                    let distance_sq = dx * dx + dy * dy;
                    if distance_sq > radius_sq {
                        continue;
                    }

                    let weight = 1.0 / (1.0 + distance_sq as f32);
                    let offset = pixel_offset(nx, ny, width);
                    for channel in 0..4 {
                        totals[channel] += source[offset + channel] as f32 * weight;
                    }
                    total_weight += weight;
                }
            }

            let offset = pixel_offset(x, y, width);
            if total_weight == 0.0 {
                destination[offset..offset + 4].copy_from_slice(&source[offset..offset + 4]);
                continue;
            }

            for channel in 0..4 {
                destination[offset + channel] =
                    (totals[channel] / total_weight).round().clamp(0.0, 255.0) as u8;
            }
        }
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

fn parse_params(params: *const c_char) -> BlurParams {
    if params.is_null() {
        return BlurParams::default();
    }

    let params = unsafe { CStr::from_ptr(params) };
    match params.to_str() {
        Ok(raw) if raw.trim().is_empty() => BlurParams::default(),
        Ok(raw) => serde_json::from_str(raw).unwrap_or_default(),
        Err(_) => BlurParams::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_blur, BlurParams};

    #[test]
    fn radius_zero_keeps_image_unchanged() {
        let mut buffer = pixels([[10, 20, 30, 255], [40, 50, 60, 255]]);
        let original = buffer.clone();

        apply_blur(
            &mut buffer,
            2,
            1,
            BlurParams {
                radius: 0,
                iterations: 3,
            },
        );

        assert_eq!(buffer, original);
    }

    #[test]
    fn zero_iterations_keep_image_unchanged() {
        let mut buffer = pixels([[10, 20, 30, 255], [40, 50, 60, 255]]);
        let original = buffer.clone();

        apply_blur(
            &mut buffer,
            2,
            1,
            BlurParams {
                radius: 1,
                iterations: 0,
            },
        );

        assert_eq!(buffer, original);
    }

    #[test]
    fn blur_softens_neighboring_pixels() {
        let mut buffer = pixels([
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [0, 0, 0, 255],
        ]);

        apply_blur(
            &mut buffer,
            3,
            1,
            BlurParams {
                radius: 1,
                iterations: 1,
            },
        );

        assert_eq!(&buffer[3..4], &[255]);
        assert_eq!(&buffer[7..8], &[255]);
        assert_eq!(&buffer[11..12], &[255]);
        assert!(buffer[0] > 0);
        assert!(buffer[4] < 255);
        assert!(buffer[8] > 0);
    }

    fn pixels<const N: usize>(pixels: [[u8; 4]; N]) -> Vec<u8> {
        pixels.into_iter().flat_map(|pixel| pixel.into_iter()).collect()
    }
}
