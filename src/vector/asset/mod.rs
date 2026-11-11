/// The [`AssetLoader`](bevy::asset::AssetLoader) for [`SvgAsset`]s.
pub mod loader;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Transform, Tree},
};
use std::{
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// An [`Asset`] containing an [`SVG`](https://en.wikipedia.org/wiki/SVG) file
/// deserialised visually losslessly[^1] as a [`Tree`] container.
///
/// [^1]: Only visually lossless for the static SVG subset.
#[derive(TypePath, Asset)]
pub struct SvgAsset(pub(crate) Arc<Tree>);

#[derive(Debug, Clone, Copy)]
pub(crate) struct IntSize {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

impl IntSize {
    const fn dimensions(self) -> (u32, u32) {
        (self.width.get(), self.height.get())
    }
}

impl Default for IntSize {
    fn default() -> Self {
        Self {
            width: NonZeroU32::new(1).unwrap(),
            height: NonZeroU32::new(1).unwrap(),
        }
    }
}

/// Rasterises a [`Tree`] into an [`Image`] of exactly [`size`](IntSize) pixels.
///
/// _May_ return [`None`] if `cancelled` was set before the iteration finished.
///
/// ## Cancellation
///
/// Cancellation is checked before and after the complete resvg render. A running
/// render finishes on its worker, but its cancelled result is never installed.
pub(crate) fn rasterise(
    tree: &Tree,
    size: IntSize,
    cancel: &AtomicBool,
    samples: u32,
) -> Option<Image> {
    let (width, height) = size.dimensions();
    let raster_width = width.checked_mul(samples)?;
    let raster_height = height.checked_mul(samples)?;
    let original_size = tree.size();
    let (original_width, original_height) = (original_size.width(), original_size.height());

    #[expect(
        clippy::cast_precision_loss,
        reason = "Irrelevant, we could just as well have used f16"
    )]
    let (sx, sy) = (
        raster_width as f32 / original_width,
        raster_height as f32 / original_height,
    );
    let transform = Transform::from_scale(sx, sy);

    let mut pixmap = Pixmap::new(raster_width, raster_height)?;
    if cancel.load(Ordering::Relaxed) {
        return None;
    }
    // Render the complete SVG. Per-node layer bounds can be empty for stroked
    // horizontal/vertical paths, despite those paths having visible geometry.
    resvg::render(tree, transform, &mut pixmap.as_mut());
    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    let pixmap = if samples > 1 {
        let mut resolved = Pixmap::new(width, height)?;
        resolved.draw_pixmap(
            0,
            0,
            pixmap.as_ref(),
            &resvg::tiny_skia::PixmapPaint {
                quality: resvg::tiny_skia::FilterQuality::Bilinear,
                ..Default::default()
            },
            Transform::from_scale(1.0 / samples as f32, 1.0 / samples as f32),
            None,
        );
        resolved
    } else {
        pixmap
    };
    let size = Extent3d {
        width: pixmap.width(),
        height: pixmap.height(),
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        pixmap.take_demultiplied(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    Some(image)
}
