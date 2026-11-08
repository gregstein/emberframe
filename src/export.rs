//! Rasterizes a page to PNG, and measures text with the same font used for export.

use std::path::Path;
use std::sync::OnceLock;

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use anyhow::{anyhow, Context, Result};
use tiny_skia::{
    BlendMode, ColorU8, FillRule, FilterQuality, IntRect, LineCap, LineJoin, Paint, PathBuilder, Pixmap, PixmapPaint,
    PremultipliedColorU8, Stroke as SkStroke, Transform,
};

use crate::bitmap::Bitmap;
use crate::model::{Color, Object, Page, Rect, Shape, Vec2, WebKind};

pub const LINE_HEIGHT: f32 = 1.2;

fn font() -> Option<&'static FontVec> {
    static FONT: OnceLock<Option<FontVec>> = OnceLock::new();
    FONT.get_or_init(|| {
        let bytes = std::fs::read("C:/Windows/Fonts/segoeui.ttf").ok()?;
        FontVec::try_from_vec(bytes).ok()
    })
    .as_ref()
}

/// Converts an em size, as used by the canvas, to the font's pixel-height scale.
fn em_scale(font: &FontVec, size: f32) -> PxScale {
    match font.units_per_em() {
        Some(units) => PxScale::from(size * font.height_unscaled() / units),
        None => PxScale::from(size),
    }
}

/// Returns the width and height of multi-line text at `size` pixels.
pub fn measure_text(content: &str, size: f32) -> (f32, f32) {
    let line_count = content.split('\n').count() as f32;
    let width = content
        .split('\n')
        .map(|line| match font() {
            Some(f) => {
                let scaled = f.as_scaled(em_scale(f, size));
                line.chars().map(|c| scaled.h_advance(scaled.glyph_id(c))).sum::<f32>()
            }
            None => line.chars().count() as f32 * size * 0.55,
        })
        .fold(0.0, f32::max);
    (width.max(size * 0.5), line_count * size * LINE_HEIGHT)
}

fn sk_paint(color: Color, opacity: f32) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(color.r, color.g, color.b, (color.a as f32 * opacity) as u8);
    paint
}

fn sk_path(points: &[Vec2], closed: bool) -> Option<tiny_skia::Path> {
    let (first, rest) = points.split_first()?;
    let mut builder = PathBuilder::new();
    builder.move_to(first.x, first.y);
    for p in rest {
        builder.line_to(p.x, p.y);
    }
    if closed {
        builder.close();
    }
    builder.finish()
}

fn blend_pixel(pixmap: &mut Pixmap, x: i32, y: i32, color: Color, coverage: f32) {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    if x < 0 || y < 0 || x >= w || y >= h {
        return;
    }
    let alpha = (color.a as f32 / 255.0 * coverage).clamp(0.0, 1.0);
    let index = (y * w + x) as usize;
    let dst = pixmap.pixels()[index];
    let over = |src: u8, dst: u8| (src as f32 * alpha + dst as f32 * (1.0 - alpha)).round() as u8;
    let a = (255.0 * alpha + dst.alpha() as f32 * (1.0 - alpha)).round() as u8;
    let min = |c: u8| c.min(a);
    if let Some(px) = PremultipliedColorU8::from_rgba(
        min(over(color.r, dst.red())),
        min(over(color.g, dst.green())),
        min(over(color.b, dst.blue())),
        a,
    ) {
        pixmap.pixels_mut()[index] = px;
    }
}

fn draw_text(pixmap: &mut Pixmap, object: &Object, content: &str, size: f32) {
    let Some(font) = font() else { return };
    let Some(fill) = object.fill else { return };
    let color = Color { a: (fill.a as f32 * object.opacity) as u8, ..fill };
    let scale = em_scale(font, size);
    let scaled = font.as_scaled(scale);
    let line_height = size * LINE_HEIGHT;
    let lead = (line_height - (scaled.ascent() - scaled.descent())) / 2.0;
    for (row, line) in content.split('\n').enumerate() {
        let mut caret = object.bounds.x;
        let baseline = object.bounds.y + row as f32 * line_height + lead + scaled.ascent();
        for ch in line.chars() {
            let id = scaled.glyph_id(ch);
            let glyph = id.with_scale_and_position(scale, ab_glyph::point(caret, baseline));
            if let Some(outlined) = font.outline_glyph(glyph) {
                let b = outlined.px_bounds();
                outlined.draw(|gx, gy, coverage| {
                    blend_pixel(pixmap, b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32, color, coverage);
                });
            }
            caret += scaled.h_advance(id);
        }
    }
}

fn draw_bitmap(target: &mut Pixmap, object: &Object, bitmap: &Bitmap) {
    let Some(mut source) = Pixmap::new(bitmap.width(), bitmap.height()) else { return };
    for (dst, px) in source.pixels_mut().iter_mut().zip(bitmap.rgba().chunks_exact(4)) {
        *dst = ColorU8::from_rgba(px[0], px[1], px[2], px[3]).premultiply();
    }
    let b = object.bounds;
    let (sx, sy) = (b.w / bitmap.width() as f32, b.h / bitmap.height() as f32);
    let quality = if sx == 1.0 && sy == 1.0 && b.x.fract() == 0.0 && b.y.fract() == 0.0 {
        FilterQuality::Nearest
    } else {
        FilterQuality::Bilinear
    };
    let paint = PixmapPaint { opacity: object.opacity, blend_mode: BlendMode::SourceOver, quality };
    target.draw_pixmap(0, 0, source.as_ref(), &paint, Transform::from_row(sx, 0.0, 0.0, sy, b.x, b.y), None);
}

fn draw_object(pixmap: &mut Pixmap, object: &Object) {
    if !object.visible || object.is_web() {
        return;
    }
    if object.is_instance() {
        for part in object.instance_parts() {
            draw_object(pixmap, &part);
        }
        return;
    }
    if let Shape::Text { content, size } = &object.shape {
        draw_text(pixmap, object, content, *size);
        return;
    }
    if let Shape::Image { bitmap } = &object.shape {
        draw_bitmap(pixmap, object, bitmap);
        return;
    }
    if let Some(fill) = object.fill {
        if let Some(path) = sk_path(&object.outline(), object.is_closed()) {
            pixmap.fill_path(&path, &sk_paint(fill, object.opacity), FillRule::EvenOdd, Transform::identity(), None);
        }
    }
    if let Some(stroke) = object.stroke {
        let style = SkStroke {
            width: stroke.width.max(0.1),
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..SkStroke::default()
        };
        for (run, closed) in object.stroke_runs() {
            if let Some(path) = sk_path(&run, closed) {
                pixmap.stroke_path(&path, &sk_paint(stroke.color, object.opacity), &style, Transform::identity(), None);
            }
        }
    }
}

pub fn render_page(page: &Page) -> Result<Pixmap> {
    let mut pixmap = Pixmap::new(page.width.max(1), page.height.max(1))
        .ok_or_else(|| anyhow!("canvas size {}x{} is too large", page.width, page.height))?;
    if let Some(c) = page.canvas {
        pixmap.fill(tiny_skia::Color::from_rgba8(c.r, c.g, c.b, c.a));
    }
    for layer in page.layers.iter().filter(|l| l.visible) {
        for object in &layer.objects {
            draw_object(&mut pixmap, object);
        }
    }
    Ok(pixmap)
}

/// Writes the page to `path` in the format named by its extension.
pub fn export_image(page: &Page, path: &Path) -> Result<()> {
    save_pixmap(&render_page(page)?, path)
}

/// Writes slice images and a hotspot image map beside an exported image.
/// Returns how many extra files were written.
pub fn export_web(page: &Page, path: &Path) -> Result<usize> {
    let shapes: Vec<(&Object, WebKind, &str)> = page
        .layers
        .iter()
        .filter(|l| l.visible)
        .flat_map(|l| l.objects.iter())
        .filter(|o| o.visible)
        .filter_map(|o| match &o.shape {
            Shape::Web { kind, url } => Some((o, *kind, url.as_str())),
            _ => None,
        })
        .collect();
    if shapes.is_empty() {
        return Ok(0);
    }
    let stem = path.file_stem().map_or_else(|| "export".into(), |s| s.to_string_lossy().into_owned());
    let extension = path.extension().map_or_else(|| "png".into(), |e| e.to_string_lossy().into_owned());
    let pixmap = render_page(page)?;
    let mut written = 0;
    for (object, _, _) in shapes.iter().filter(|(_, kind, _)| *kind == WebKind::Slice) {
        let Some(area) = page.rect().intersect(object.bounds) else { continue };
        let crop = IntRect::from_xywh(area.x.round() as i32, area.y.round() as i32, area.w.round().max(1.0) as u32, area.h.round().max(1.0) as u32);
        let Some(piece) = crop.and_then(|c| pixmap.clone_rect(c)) else { continue };
        written += 1;
        save_pixmap(&piece, &path.with_file_name(format!("{stem}_slice{written}.{extension}")))?;
    }
    let hotspots: Vec<_> = shapes.iter().filter(|(_, kind, _)| *kind == WebKind::Hotspot).collect();
    if !hotspots.is_empty() {
        let image = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let html = image_map(&image, page, hotspots.iter().map(|(object, _, url)| (object.bounds, object.name.as_str(), *url)));
        let target = path.with_file_name(format!("{stem}_map.html"));
        std::fs::write(&target, html).with_context(|| format!("could not write {}", target.display()))?;
        written += 1;
    }
    Ok(written)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}

fn image_map<'a>(image: &str, page: &Page, areas: impl Iterator<Item = (Rect, &'a str, &'a str)>) -> String {
    let mut html = format!(
        "<!doctype html>\n<img src=\"{}\" width=\"{}\" height=\"{}\" usemap=\"#map\" alt=\"\">\n<map name=\"map\">\n",
        escape_html(image),
        page.width,
        page.height
    );
    for (b, name, url) in areas {
        let href = if url.is_empty() { "#" } else { url };
        html.push_str(&format!(
            "  <area shape=\"rect\" coords=\"{},{},{},{}\" href=\"{}\" alt=\"{}\">\n",
            b.x.round() as i32,
            b.y.round() as i32,
            b.right().round() as i32,
            b.bottom().round() as i32,
            escape_html(href),
            escape_html(name)
        ));
    }
    html.push_str("</map>\n");
    html
}

fn save_pixmap(pixmap: &Pixmap, path: &Path) -> Result<()> {
    let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    if extension == "png" {
        return pixmap.save_png(path).with_context(|| format!("could not write {}", path.display()));
    }
    let mut rgba = Vec::with_capacity(pixmap.data().len());
    for px in pixmap.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    let image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), rgba)
        .ok_or_else(|| anyhow!("rendered image has an invalid size"))?;
    let result = if matches!(extension.as_str(), "jpg" | "jpeg") {
        flatten_on_white(&image).save(path)
    } else {
        image.save(path)
    };
    result.with_context(|| format!("could not write {}", path.display()))
}

/// JPEG has no alpha channel, so transparency is composited over white.
fn flatten_on_white(image: &image::RgbaImage) -> image::RgbImage {
    let (w, h) = image.dimensions();
    image::RgbImage::from_fn(w, h, |x, y| {
        let [r, g, b, a] = image.get_pixel(x, y).0;
        let alpha = a as f32 / 255.0;
        let over = |c: u8| (c as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8;
        image::Rgb([over(r), over(g), over(b)])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Rect, Stroke};

    fn page() -> Page {
        Page::new("t", 40, 40)
    }

    fn rgba(p: &Pixmap, x: u32, y: u32) -> (u8, u8, u8, u8) {
        let c = p.pixel(x, y).unwrap().demultiply();
        (c.red(), c.green(), c.blue(), c.alpha())
    }

    fn red_box(x: f32, w: f32) -> Object {
        Object {
            id: 1,
            name: "box".into(),
            shape: Shape::Rect { radius: 0.0 },
            bounds: Rect { x, y: 0.0, w, h: 40.0 },
            points: vec![],
            fill: Some(Color::rgb(255, 0, 0)),
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    #[test]
    fn fills_canvas_and_object() {
        let mut p = page();
        p.layers[0].objects.push(red_box(0.0, 20.0));
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 5, 20), (255, 0, 0, 255));
        assert_eq!(rgba(&img, 30, 20), (255, 255, 255, 255));
    }

    #[test]
    fn transparent_canvas_stays_transparent() {
        let mut p = page();
        p.canvas = None;
        assert_eq!(rgba(&render_page(&p).unwrap(), 10, 10).3, 0);
    }

    #[test]
    fn hidden_layers_and_objects_are_not_exported() {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.visible = false;
        p.layers[0].objects.push(o);
        assert_eq!(rgba(&render_page(&p).unwrap(), 5, 5), (255, 255, 255, 255));
        p.layers[0].objects[0].visible = true;
        p.layers[0].visible = false;
        assert_eq!(rgba(&render_page(&p).unwrap(), 5, 5), (255, 255, 255, 255));
    }

    #[test]
    fn opacity_blends_with_canvas() {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.opacity = 0.5;
        p.layers[0].objects.push(o);
        let (r, g, _, a) = rgba(&render_page(&p).unwrap(), 20, 20);
        assert_eq!((r, a), (255, 255));
        assert!((120..=135).contains(&g), "g was {g}");
    }

    #[test]
    fn stroke_is_painted_on_open_path() {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.shape = Shape::Line;
        o.fill = None;
        o.points = vec![crate::model::v2(0.0, 20.0), crate::model::v2(40.0, 20.0)];
        o.stroke = Some(Stroke { color: Color::BLACK, width: 4.0 });
        p.layers[0].objects.push(o);
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 20, 20), (0, 0, 0, 255));
        assert_eq!(rgba(&img, 20, 5), (255, 255, 255, 255));
    }

    #[test]
    fn png_file_is_written_with_page_size() {
        let path = std::env::temp_dir().join("emberframe_export_test.png");
        export_image(&page(), &path).unwrap();
        let img = Pixmap::load_png(&path).unwrap();
        assert_eq!((img.width(), img.height()), (40, 40));
        std::fs::remove_file(path).ok();
    }

    fn green_bitmap() -> Object {
        let pixels = [0u8, 255, 0, 255].repeat(20 * 20);
        Object::new_bitmap(9, "g", Bitmap::from_rgba(20, 20, pixels).unwrap(), 10.0, 10.0)
    }

    #[test]
    fn bitmap_is_drawn_at_its_bounds() {
        let mut p = page();
        p.layers[0].objects.push(green_bitmap());
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 20, 20), (0, 255, 0, 255));
        assert_eq!(rgba(&img, 5, 5), (255, 255, 255, 255));
    }

    #[test]
    fn scaled_bitmap_covers_scaled_bounds() {
        let mut p = page();
        let mut o = green_bitmap();
        o.set_bounds(Rect { x: 0.0, y: 0.0, w: 40.0, h: 40.0 });
        p.layers[0].objects.push(o);
        assert_eq!(rgba(&render_page(&p).unwrap(), 36, 36), (0, 255, 0, 255));
    }

    #[test]
    fn jpeg_export_flattens_transparency_on_white() {
        let path = std::env::temp_dir().join("emberframe_export_test.jpg");
        let mut p = page();
        p.canvas = None;
        export_image(&p, &path).unwrap();
        let decoded = image::open(&path).unwrap().to_rgb8();
        assert!(decoded.get_pixel(20, 20).0.iter().all(|c| *c > 250));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn web_areas_stay_out_of_the_image_but_produce_files() {
        let dir = std::env::temp_dir().join("emberframe_web_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("site.png");
        let mut p = page();
        let before = rgba(&render_page(&p).unwrap(), 5, 5);
        p.layers[0].objects.push(Object::new_web(7, WebKind::Slice, Rect { x: 0.0, y: 0.0, w: 10.0, h: 20.0 }));
        let mut hotspot = Object::new_web(8, WebKind::Hotspot, Rect { x: 5.0, y: 6.0, w: 10.0, h: 10.0 });
        hotspot.shape = Shape::Web { kind: WebKind::Hotspot, url: "https://example.com/?a=1&b=2".into() };
        p.layers[0].objects.push(hotspot);
        assert_eq!(rgba(&render_page(&p).unwrap(), 5, 5), before);
        assert_eq!(export_web(&p, &path).unwrap(), 2);
        let slice = Pixmap::load_png(dir.join("site_slice1.png")).unwrap();
        assert_eq!((slice.width(), slice.height()), (10, 20));
        let html = std::fs::read_to_string(dir.join("site_map.html")).unwrap();
        assert!(html.contains("coords=\"5,6,15,16\""));
        assert!(html.contains("a=1&amp;b=2"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn filled_open_path_keeps_its_fill() {
        let mut p = page();
        let mut piece = Object::new_web(9, WebKind::Slice, Rect::default());
        piece.shape = Shape::Path { closed: false };
        piece.points = vec![
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 39.0, y: 0.0 },
            Vec2 { x: 39.0, y: 39.0 },
            Vec2 { x: 0.0, y: 39.0 },
        ];
        piece.fill = Some(Color::rgb(0, 255, 0));
        p.layers[0].objects.push(piece);
        assert_eq!(rgba(&render_page(&p).unwrap(), 20, 20), (0, 255, 0, 255));
    }

    #[test]
    fn no_web_areas_means_no_extra_files() {
        assert_eq!(export_web(&page(), Path::new("unused.png")).unwrap(), 0);
    }

    #[test]
    fn unknown_extension_reports_an_error() {
        let path = std::env::temp_dir().join("emberframe_export_test.nope");
        assert!(export_image(&page(), &path).is_err());
    }

    #[test]
    fn text_is_measured_at_em_size() {
        let Some(f) = font() else { return };
        let id = f.glyph_id('H');
        let upem = f.units_per_em().unwrap();
        let expected = 4.0 * f.h_advance_unscaled(id) / upem * 100.0;
        let (width, _) = measure_text("HHHH", 100.0);
        assert!((width - expected).abs() < 0.5, "width {width}, expected {expected}");
    }

    #[test]
    fn measure_grows_with_lines() {
        let (_, one) = measure_text("a", 10.0);
        let (_, two) = measure_text("a\nb", 10.0);
        assert!(two > one);
    }
}
