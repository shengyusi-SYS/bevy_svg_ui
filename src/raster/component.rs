use crate::raster::asset::SvgFile;
use bevy::prelude::*;

/// A color tint to apply to an SVG when rendering.
/// This will be multiplied with the SVG's original colors.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct SvgColor(pub Color);

impl SvgColor {
    /// Creates a new [`SvgColor`] with the specified color.
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    /// White color (no tint, original colors).
    pub const WHITE: Self = Self(Color::WHITE);
}

impl From<Color> for SvgColor {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

/// Determines how the SVG color should be obtained.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub enum SvgColorMode {
    /// Use the explicit [`SvgColor`] component value.
    Explicit,
    /// Inherit color from the entity's own [`TextColor`] component.
    /// This is the default mode, allowing SVGs to automatically match text color
    /// when a [`TextColor`] component is present on the same entity.
    /// If no [`TextColor`] is found, the original [`SvgColor`] is preserved.
    #[default]
    InheritTextColor,
}

/// Determines how the SVG should be scaled for UI rendering.
#[derive(Component, Clone, Debug, PartialEq)]
pub enum SvgScaleMode {
    /// Use the original SVG size (viewBox dimensions).
    Original,
    /// Scale to fit the computed Node size while maintaining aspect ratio.
    /// The SVG will be rendered at the resolution needed to fill the container.
    FitNode,
    /// Use a custom scale factor for rendering.
    /// A value of 2.0 means render at 2x resolution for sharper display.
    Custom(f32),
}
impl Default for SvgScaleMode {
    fn default() -> Self {
        SvgScaleMode::Custom(4.0)
    }
}

/// The [`Component`] for rendering an SVG as a 2D sprite.
/// Use this when you want to display the SVG in world space as a [`Sprite`].
#[derive(Component, Default)]
#[require(SvgColor)]
pub struct Svg2d(pub Handle<SvgFile>);

impl Svg2d {
    /// Creates a new [`Svg2d`] component with the given SVG file handle.
    pub fn new(handle: Handle<SvgFile>) -> Self {
        Self(handle)
    }
}

/// The [`Component`] for rendering an SVG as a UI element.
/// Use this when you want to display the SVG within the UI hierarchy,
/// supporting flex layout and other UI features.
///
/// This component will result in an [`ImageNode`] being added to the entity,
/// making it compatible with Bevy's UI system.
#[derive(Component, Default, Clone)]
#[require(SvgColor, SvgColorMode, SvgScaleMode, Node)]
pub struct SvgUi {
    /// The handle to the SVG file asset.
    pub handle: Handle<SvgFile>,
}

impl SvgUi {
    /// Creates a new [`SvgUi`] component with the given SVG file handle.
    pub fn new(handle: Handle<SvgFile>) -> Self {
        Self { handle }
    }

    /// Creates an [`SvgUi`] that inherits its color from the entity's [`TextColor`].
    pub fn with_text_color(handle: Handle<SvgFile>) -> (Self, SvgColorMode) {
        (Self::new(handle), SvgColorMode::InheritTextColor)
    }
}

impl From<Handle<SvgFile>> for SvgUi {
    fn from(handle: Handle<SvgFile>) -> Self {
        Self::new(handle)
    }
}

impl From<Handle<SvgFile>> for Svg2d {
    fn from(handle: Handle<SvgFile>) -> Self {
        Self::new(handle)
    }
}
