/// The [`AssetLoader`](bevy::asset::AssetLoader) for [`SvgVectorAsset`]s.
pub mod loader;

use crate::error::{Result, SvgError};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use regex_lite::Regex;
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree},
};
use std::sync::Arc;

/// An [`Asset`] containing an [`SVG`](https://en.wikipedia.org/wiki/SVG) file
/// with its original data preserved for color modification.
///
/// The original SVG bytes are stored to allow runtime modification of
/// stroke and fill attributes before re-rendering.
#[derive(TypePath, Asset)]
pub struct SvgVectorAsset {
    /// The original SVG file data as bytes.
    pub data: Arc<Vec<u8>>,
    /// The parsed usvg Tree for the original (unmodified) SVG.
    tree: Tree,
}

impl SvgVectorAsset {
    /// Creates a new [`SvgVectorAsset`] from raw SVG data.
    ///
    /// ## Errors
    ///
    /// Returns an error if the SVG data cannot be parsed.
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let options = Options::default();
        let tree = Tree::from_data(&data, &options)?;
        Ok(Self {
            data: Arc::new(data),
            tree,
        })
    }

    /// Modifies SVG content by replacing stroke and fill attributes with a new color.
    ///
    /// This function modifies the raw SVG data to replace all stroke and fill
    /// color values with the specified color. The modified SVG is then parsed
    /// into a new Tree for rendering.
    fn create_colored_tree(&self, color: Color) -> Result<Tree> {
        let linear = color.to_linear();
        // Convert to sRGB hex color for SVG
        let r = (linear.red.powf(1.0 / 2.2) * 255.0).round() as u8;
        let g = (linear.green.powf(1.0 / 2.2) * 255.0).round() as u8;
        let b = (linear.blue.powf(1.0 / 2.2) * 255.0).round() as u8;
        let hex_color = format!("#{:02x}{:02x}{:02x}", r, g, b);

        let svg_str = String::from_utf8_lossy(&self.data);
        let modified_svg = modify_svg_colors(&svg_str, &hex_color, linear.alpha);

        let options = Options::default();
        let tree = Tree::from_data(modified_svg.as_bytes(), &options)?;
        Ok(tree)
    }

    /// Renders and rasterises an [`SvgVectorAsset`] into a [`Pixmap`].
    ///
    /// The rendered [`Pixmap`] is the same size as the SVG file's
    /// [`viewBox`](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute).
    ///
    /// ## Errors
    ///
    /// The `viewBox` *must not* be 0 on any axis. If this invariant is broken,
    /// then this function will return an [`SvgError::Empty`].
    fn render(&self) -> Result<Pixmap> {
        let (width, height) = self.tree.size().to_int_size().dimensions();
        let mut buf = Pixmap::new(width, height).ok_or(SvgError::Empty)?;

        resvg::render(&self.tree, Transform::default(), &mut buf.as_mut());
        Ok(buf)
    }

    /// Renders a specific tree into a [`Pixmap`] at the given size.
    fn render_tree_at_size(tree: &Tree, target_size: UVec2) -> Result<Pixmap> {
        let original_size = tree.size();
        let scale_x = target_size.x as f32 / original_size.width();
        let scale_y = target_size.y as f32 / original_size.height();

        let mut pixmap = Pixmap::new(target_size.x, target_size.y).ok_or(SvgError::Empty)?;
        let transform = Transform::from_scale(scale_x, scale_y);
        resvg::render(tree, transform, &mut pixmap.as_mut());
        Ok(pixmap)
    }

    /// Renders and rasterises an [`SvgVectorAsset`] into an [`Image`].
    ///
    /// The rendered [`Image`] is the same size as the SVG file's
    /// [`viewBox`](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute).
    ///
    /// ## Errors
    ///
    /// The `viewBox` *must not* be 0 on any axis. If this invariant is broken,
    /// then this function will return an [`SvgError::Empty`].
    pub fn render_to_image(&self, asset_usage: RenderAssetUsages) -> Result<Image> {
        let pixmap = self.render()?;
        let (width, height) = self.tree.size().to_int_size().dimensions();
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        Ok(Image::new(
            size,
            TextureDimension::D2,
            pixmap.take(),
            TextureFormat::Rgba8Unorm,
            asset_usage,
        ))
    }

    /// Renders and rasterises an [`SvgVectorAsset`] into an [`Image`] with a color applied.
    ///
    /// This method modifies the SVG's stroke and fill attributes to the specified color
    /// before rendering. This approach preserves the SVG's structure and produces
    /// higher-quality results than post-processing.
    ///
    /// ## Errors
    ///
    /// The `viewBox` *must not* be 0 on any axis. If this invariant is broken,
    /// then this function will return an [`SvgError::Empty`].
    pub fn render_to_image_with_color(
        &self,
        color: Color,
        asset_usage: RenderAssetUsages,
    ) -> Result<Image> {
        // If color is white, use the original rendering
        if color == Color::WHITE {
            return self.render_to_image(asset_usage);
        }

        let colored_tree = self.create_colored_tree(color)?;
        let (width, height) = colored_tree.size().to_int_size().dimensions();
        let mut pixmap = Pixmap::new(width, height).ok_or(SvgError::Empty)?;

        resvg::render(&colored_tree, Transform::default(), &mut pixmap.as_mut());

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        Ok(Image::new(
            size,
            TextureDimension::D2,
            pixmap.take(),
            TextureFormat::Rgba8Unorm,
            asset_usage,
        ))
    }

    /// Returns the original size of the SVG (from viewBox).
    pub fn original_size(&self) -> UVec2 {
        let (width, height) = self.tree.size().to_int_size().dimensions();
        UVec2::new(width, height)
    }

    /// Renders the SVG at a specific target size with optional color.
    ///
    /// This method scales the SVG to fit the target size, which is useful for
    /// UI rendering where the container size may differ from the original SVG size.
    ///
    /// ## Arguments
    ///
    /// * `target_size` - The target size in pixels to render the SVG at.
    /// * `color` - Optional color to apply. Pass `None` or `Some(Color::WHITE)` for original colors.
    /// * `asset_usage` - The asset usage flags for the resulting image.
    ///
    /// ## Errors
    ///
    /// Returns an error if the SVG cannot be rendered or the target size is zero.
    pub fn render_to_image_at_size(
        &self,
        target_size: UVec2,
        color: Option<Color>,
        asset_usage: RenderAssetUsages,
    ) -> Result<Image> {
        if target_size.x == 0 || target_size.y == 0 {
            return Err(SvgError::Empty);
        }

        let pixmap = match color {
            Some(c) if c != Color::WHITE => {
                let colored_tree = self.create_colored_tree(c)?;
                Self::render_tree_at_size(&colored_tree, target_size)?
            }
            _ => Self::render_tree_at_size(&self.tree, target_size)?,
        };

        let size = Extent3d {
            width: target_size.x,
            height: target_size.y,
            depth_or_array_layers: 1,
        };
        Ok(Image::new(
            size,
            TextureDimension::D2,
            pixmap.take(),
            TextureFormat::Rgba8Unorm,
            asset_usage,
        ))
    }
}

/// Modifies SVG content by replacing stroke and fill color values.
///
/// This function uses simple string replacement to modify color attributes
/// in the SVG. It handles:
/// - `fill="..."` attributes
/// - `stroke="..."` attributes  
/// - `fill:...` and `stroke:...` in style attributes
/// - CSS color values (hex, rgb, named colors)
///
/// Note: This is a simple implementation that works well for monochrome icons.
/// Complex SVGs with gradients or multiple intentional colors may need
/// more sophisticated handling.
fn modify_svg_colors(svg: &str, new_color: &str, alpha: f32) -> String {
    let mut result = svg.to_string();

    // List of color patterns to replace
    // We match common color formats: hex, rgb, rgba, and named colors
    let color_patterns = [
        // Hex colors
        r#"#[0-9a-fA-F]{6}"#,
        r#"#[0-9a-fA-F]{3}"#,
        // RGB/RGBA
        r#"rgb\([^)]+\)"#,
        r#"rgba\([^)]+\)"#,
        // Common named colors often used in icons
        "black",
        "white",
        "currentColor",
    ];

    // Replace fill attributes
    for pattern in &color_patterns {
        // Handle fill="color" and stroke="color" attributes
        let fill_regex = format!(r#"fill="{}""#, pattern);
        let stroke_regex = format!(r#"stroke="{}""#, pattern);

        if let Ok(re) = Regex::new(&fill_regex) {
            result = re
                .replace_all(&result, format!(r#"fill="{}""#, new_color).as_str())
                .to_string();
        }
        if let Ok(re) = Regex::new(&stroke_regex) {
            result = re
                .replace_all(&result, format!(r#"stroke="{}""#, new_color).as_str())
                .to_string();
        }

        // Handle style="fill:color" and style="stroke:color"
        let style_fill_regex = format!(r#"fill:\s*{}"#, pattern);
        let style_stroke_regex = format!(r#"stroke:\s*{}"#, pattern);

        if let Ok(re) = Regex::new(&style_fill_regex) {
            result = re
                .replace_all(&result, format!("fill:{}", new_color).as_str())
                .to_string();
        }
        if let Ok(re) = Regex::new(&style_stroke_regex) {
            result = re
                .replace_all(&result, format!("stroke:{}", new_color).as_str())
                .to_string();
        }
    }

    // Handle opacity if alpha is not 1.0
    if alpha < 1.0 {
        let opacity_str = format!("{:.2}", alpha);
        // Add or replace fill-opacity and stroke-opacity
        // For simplicity, we add a global opacity if the root has a style
        if result.contains("fill-opacity") {
            if let Ok(re) = Regex::new(r#"fill-opacity="[^"]*""#) {
                result = re
                    .replace_all(
                        &result,
                        format!(r#"fill-opacity="{}""#, opacity_str).as_str(),
                    )
                    .to_string();
            }
        }
        if result.contains("stroke-opacity") {
            if let Ok(re) = Regex::new(r#"stroke-opacity="[^"]*""#) {
                result = re
                    .replace_all(
                        &result,
                        format!(r#"stroke-opacity="{}""#, opacity_str).as_str(),
                    )
                    .to_string();
            }
        }
    }

    result
}
