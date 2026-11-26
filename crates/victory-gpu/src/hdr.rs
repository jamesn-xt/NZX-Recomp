//! Colour conversion at the boundary between RGB9E5 title memory and
//! RGBA16_FLOAT host targets. The shared exponent represents 2^(e - 24)
//! times each unsigned nine-bit mantissa; the missing alpha samples as 1.
use crate::primitive::{f32_to_half, half_to_f32};

pub(crate) fn expand_row(packed: &[u8], expanded: &mut [u8]) {
    debug_assert_eq!(packed.len() * 2, expanded.len());
    for (pixel, out) in packed.as_chunks::<4>().0.iter().zip(expanded.as_chunks_mut::<8>().0) {
        let pixel = u32::from_le_bytes(*pixel);
        let scale = 2f32.powi((pixel >> 27) as i32 - 24);
        for (i, shift) in [0, 9, 18].into_iter().enumerate() {
            let channel = ((pixel >> shift) & 511) as f32 * scale;
            out[i * 2..i * 2 + 2].copy_from_slice(&f32_to_half(channel).to_le_bytes());
        }
        out[6..].copy_from_slice(&0x3c00u16.to_le_bytes());
    }
}

pub(crate) fn pack_row(expanded: &[u8], packed: &mut [u8]) {
    debug_assert_eq!(packed.len() * 2, expanded.len());
    for (pixel, out) in expanded.as_chunks::<8>().0.iter().zip(packed.as_chunks_mut::<4>().0) {
        let rgb = [0, 2, 4].map(|i| {
            let v = half_to_f32(u16::from_le_bytes([pixel[i], pixel[i + 1]]));
            if v.is_nan() { 0.0 } else { v.clamp(0.0, 65408.0) }
        });
        let max = rgb.into_iter().fold(0.0, f32::max);
        let mut e = ((max.to_bits() >> 23) as i32 - 127 + 16).max(0);
        let mut unit = 2f32.powi(e - 24);
        if (max / unit).round() == 512.0 {
            e += 1;
            unit *= 2.0;
        }
        let [r, g, b] = rgb.map(|v| (v / unit).round() as u32);
        *out = (r | (g << 9) | (b << 18) | ((e as u32) << 27)).to_le_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb9e5_round_trip_preserves_hdr_colour_and_subnormals() {
        for e in 0..32u32 {
            for i in 0..512u32 {
                // Include dark channels beside a bright one. The leading
                // 256 makes this a canonical encoding at each exponent.
                let input = (i | (256 << 9) | ((511 - i) << 18) | (e << 27)).to_le_bytes();
                let mut expanded = [0; 8];
                expand_row(&input, &mut expanded);
                assert_eq!(expanded[6..], 0x3c00u16.to_le_bytes());
                let mut packed = [0; 4];
                pack_row(&expanded, &mut packed);
                assert_eq!(packed, input);
            }
        }
    }
}
