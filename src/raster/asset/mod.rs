/// The [`AssetLoader`](bevy::asset::AssetLoader) for [`SvgFile`]s.
pub(crate) mod loader;

use crate::{error::SvgError, vector::asset::SvgVectorAsset};
use bevy::{asset::RenderAssetUsages, prelude::*};
use std::sync::Arc;

/// An [`Asset`] containing an [`SVG`](https://en.wikipedia.org/wiki/SVG) file
/// rendered into an [`Image`].
///
/// This asset stores both the rendered image and the original vector data,
/// allowing for runtime color modification and re-rendering.
#[derive(TypePath, Asset)]
pub struct SvgFile {
    /// The pre-rendered image of the SVG.
    pub image: Image,
    /// The original vector asset for re-rendering with different colors.
    /// Wrapped in Arc for cheap cloning.
    pub(crate) vector: Arc<SvgVectorAsset>,
}

impl SvgFile {
    /// Creates a new [`SvgFile`] from a vector asset.
    ///
    /// ## Errors
    ///
    /// Returns an error if the SVG cannot be rendered.
    pub fn new(vector: SvgVectorAsset, asset_usage: RenderAssetUsages) -> Result<Self, SvgError> {
        let image = vector.render_to_image(asset_usage)?;
        Ok(Self {
            image,
            vector: Arc::new(vector),
        })
    }

    /// Renders the SVG with a color tint applied.
    ///
    /// The color is multiplied with the original SVG colors.
    ///
    /// ## Errors
    ///
    /// Returns an error if the SVG cannot be rendered.
    pub fn render_with_color(
        &self,
        color: Color,
        asset_usage: RenderAssetUsages,
    ) -> Result<Image, SvgError> {
        self.vector.render_to_image_with_color(color, asset_usage)
    }

    /// Renders the SVG at a specific target size with optional color tint.
    ///
    /// This is useful for UI rendering where the SVG needs to be scaled
    /// to fit a container of a specific size.
    ///
    /// ## Arguments
    ///
    /// * `target_size` - The target size in pixels.
    /// * `color` - Optional color tint to apply.
    /// * `asset_usage` - The asset usage flags for the resulting image.
    ///
    /// ## Errors
    ///
    /// Returns an error if the SVG cannot be rendered.
    pub fn render_at_size(
        &self,
        target_size: UVec2,
        color: Option<Color>,
        asset_usage: RenderAssetUsages,
    ) -> Result<Image, SvgError> {
        self.vector
            .render_to_image_at_size(target_size, color, asset_usage)
    }

    /// Returns the original dimensions of the SVG (from viewBox).
    pub fn original_size(&self) -> UVec2 {
        self.vector.original_size()
    }

    /// Returns the dimensions of the pre-rendered image.
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.image.width(), self.image.height())
    }
}
