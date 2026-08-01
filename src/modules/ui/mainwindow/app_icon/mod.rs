/// Rasterize the SVG app icon into the window icon.
///
/// Loads `Halvora_Logo/halvoralogo.svg`, renders it at a high resolution, and
/// converts the pixels into the icon format expected by the window system.
///
/// Returns `None` when the SVG cannot be read, parsed, or rendered; the
/// application then falls back to the platform default icon.
pub fn load_app_icon() -> Option<iced::window::Icon> {
    const ICON_SIZE: u32 = 256;

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Halvora_Logo/halvoralogo.svg");
    let svg_data = std::fs::read(path).ok()?;

    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &opt).ok()?;

    let src_size = tree.size();
    let (src_w, src_h) = (src_size.width(), src_size.height());
    if src_w <= 0.0 || src_h <= 0.0 {
        return None;
    }

    // Scale the source viewbox down to the target icon size.
    let scale = ICON_SIZE as f32 / src_w.max(src_h);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE)?;
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny-skia stores premultiplied alpha; iced expects straight RGBA.
    let premult = pixmap.data();
    let mut rgba = Vec::with_capacity(premult.len());
    for px in premult.chunks_exact(4) {
        let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            rgba.extend_from_slice(&[
                (r as u16 * 255 / a as u16) as u8,
                (g as u16 * 255 / a as u16) as u8,
                (b as u16 * 255 / a as u16) as u8,
                a,
            ]);
        }
    }

    iced::window::icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).ok()
}