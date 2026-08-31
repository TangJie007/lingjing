use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP,
};

pub(crate) fn rgba_to_png_data_url(rgba: &[u8], w: u32, h: u32) -> Option<String> {
    use base64::Engine;
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&buf)
    ))
}

/// Convert a real HBITMAP menu glyph to a data URL. Skips HBMMENU_* pseudo handles.
pub(crate) unsafe fn hbitmap_to_data_url(hbmp: HBITMAP) -> Option<String> {
    let raw = hbmp.0 as isize;
    // HBMMENU_CALLBACK (-1) and system stock values (±1..16) are not real bitmaps.
    if hbmp.is_invalid() || (raw >= -16 && raw <= 16) {
        return None;
    }

    let mut bm = BITMAP::default();
    if GetObjectW(
        hbmp.into(),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bm as *mut BITMAP as *mut core::ffi::c_void),
    ) == 0
        || bm.bmWidth <= 0
        || bm.bmHeight == 0
    {
        return None;
    }
    let w = bm.bmWidth;
    let h = bm.bmHeight.abs();
    let hdc = CreateCompatibleDC(None);
    if hdc.is_invalid() {
        return None;
    }

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0u8; (w * h * 4) as usize];
    let got = GetDIBits(
        hdc,
        hbmp,
        0,
        h as u32,
        Some(bgra.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    let _ = DeleteDC(hdc);
    if got == 0 {
        return None;
    }

    let mut rgba = vec![0u8; bgra.len()];
    for (i, chunk) in bgra.chunks_exact(4).enumerate() {
        let o = i * 4;
        rgba[o] = chunk[2];
        rgba[o + 1] = chunk[1];
        rgba[o + 2] = chunk[0];
        rgba[o + 3] = chunk[3];
    }
    rgba_to_png_data_url(&rgba, w as u32, h as u32)
}

