use crate::raster::{
    asset::{SvgFile, loader::SvgFileLoader},
    component::{Svg2d, SvgColor, SvgColorMode, SvgScaleMode, SvgUi},
};
use bevy::{asset::RenderAssetUsages, prelude::*, ui::widget::ImageNode};

/// The [`Plugin`] for initialising the
/// [Rasterised](https://en.wikipedia.org/wiki/Raster_graphics)
/// [`Asset`] and [`AssetLoader`](bevy::asset::AssetLoader).
pub struct SvgRasterPlugin;

impl Plugin for SvgRasterPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SvgFile>()
            .init_asset_loader::<SvgFileLoader>()
            .add_systems(
                Update,
                (
                    spawn_svg_sprites,
                    spawn_svg_ui_nodes,
                    init_svg_sprites_from_loaded_assets,
                    init_svg_ui_from_loaded_assets,
                    update_svg_sprite_colors,
                    update_svg_ui_on_changes,
                    sync_svg_color_from_text_color,
                ),
            );
    }
}

/// Marker component to track the current rendering state of an SVG.
/// This is used to detect changes and re-render when necessary.
#[derive(Component, Default, Clone, PartialEq)]
struct AppliedSvgState {
    color: Color,
    size: UVec2,
}

/// Adds a [`Sprite`] to all [`Entities`](Entity) with an [`Svg2d`]
/// that doesn't already have an associated [`Sprite`].
fn spawn_svg_sprites(
    mut commands: Commands,
    mut svg_events: MessageReader<AssetEvent<SvgFile>>,
    svg_assets: Res<Assets<SvgFile>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &Svg2d, Option<&SvgColor>), Without<Sprite>>,
) {
    for event in svg_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                // Find all entities waiting for this SVG
                for (entity, svg_component, color) in &query {
                    if svg_component.0.id() == id.clone()
                        && let Some(svg_asset) = svg_assets.get(id.clone())
                    {
                        let tint_color = color.map(|c| c.0).unwrap_or(Color::WHITE);
                        let image = if tint_color == Color::WHITE {
                            svg_asset.image.clone()
                        } else {
                            svg_asset
                                .render_with_color(tint_color, RenderAssetUsages::default())
                                .unwrap_or_else(|_| svg_asset.image.clone())
                        };
                        let size = UVec2::new(image.width(), image.height());
                        let handle = images.add(image);

                        commands.entity(entity).insert((
                            Sprite::from_image(handle),
                            AppliedSvgState {
                                color: tint_color,
                                size,
                            },
                        ));
                    }
                }
            }
            AssetEvent::Added { id } => {
                debug!("added SVG asset '{id}'");
            }
            other => {
                // TODO: Handle more events
                debug!("Unimplemented Event for SVG raster assets: '{other:#?}'");
            }
        }
    }
}

/// Adds an [`ImageNode`] to all [`Entities`](Entity) with an [`SvgUi`]
/// that doesn't already have an associated [`ImageNode`].
fn spawn_svg_ui_nodes(
    mut commands: Commands,
    mut svg_events: MessageReader<AssetEvent<SvgFile>>,
    svg_assets: Res<Assets<SvgFile>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<
        (
            Entity,
            &SvgUi,
            Option<&SvgColor>,
            Option<&SvgScaleMode>,
            Option<&ComputedNode>,
        ),
        Without<ImageNode>,
    >,
) {
    for event in svg_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                // Find all entities waiting for this SVG
                for (entity, svg_component, color, scale_mode, computed_node) in &query {
                    if svg_component.handle.id() == id.clone()
                        && let Some(svg_asset) = svg_assets.get(id.clone())
                    {
                        let tint_color = color.map(|c| c.0).unwrap_or(Color::WHITE);

                        // Determine target size based on scale mode
                        let target_size = compute_target_size(
                            svg_asset,
                            scale_mode.unwrap_or(&SvgScaleMode::Original),
                            computed_node,
                        );

                        let image = render_svg_image(svg_asset, target_size, tint_color);
                        let rendered_size = UVec2::new(image.width(), image.height());
                        let handle = images.add(image);

                        commands.entity(entity).insert((
                            ImageNode::new(handle),
                            AppliedSvgState {
                                color: tint_color,
                                size: rendered_size,
                            },
                        ));
                    }
                }
            }
            AssetEvent::Added { id } => {
                debug!("added SVG UI asset '{id}'");
            }
            other => {
                debug!("Unimplemented Event for SVG UI assets: '{other:#?}'");
            }
        }
    }
}

/// Initializes [`Sprite`] for newly added [`Svg2d`] entities when the SVG asset is already loaded.
/// This handles the case where entities are spawned after the asset has finished loading.
fn init_svg_sprites_from_loaded_assets(
    mut commands: Commands,
    svg_assets: Res<Assets<SvgFile>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &Svg2d, Option<&SvgColor>), (Added<Svg2d>, Without<Sprite>)>,
) {
    for (entity, svg_component, color) in &query {
        if let Some(svg_asset) = svg_assets.get(&svg_component.0) {
            let tint_color = color.map(|c| c.0).unwrap_or(Color::WHITE);
            let image = if tint_color == Color::WHITE {
                svg_asset.image.clone()
            } else {
                svg_asset
                    .render_with_color(tint_color, RenderAssetUsages::default())
                    .unwrap_or_else(|_| svg_asset.image.clone())
            };
            let size = UVec2::new(image.width(), image.height());
            let handle = images.add(image);

            commands.entity(entity).insert((
                Sprite::from_image(handle),
                AppliedSvgState {
                    color: tint_color,
                    size,
                },
            ));
        }
    }
}

/// Initializes [`ImageNode`] for newly added [`SvgUi`] entities when the SVG asset is already loaded.
/// This handles the case where entities are spawned after the asset has finished loading.
fn init_svg_ui_from_loaded_assets(
    mut commands: Commands,
    svg_assets: Res<Assets<SvgFile>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<
        (
            Entity,
            &SvgUi,
            Option<&SvgColor>,
            Option<&SvgScaleMode>,
            Option<&ComputedNode>,
        ),
        (Added<SvgUi>, Without<ImageNode>),
    >,
) {
    for (entity, svg_component, color, scale_mode, computed_node) in &query {
        if let Some(svg_asset) = svg_assets.get(&svg_component.handle) {
            let tint_color = color.map(|c| c.0).unwrap_or(Color::WHITE);

            // Determine target size based on scale mode
            let target_size = compute_target_size(
                svg_asset,
                scale_mode.unwrap_or(&SvgScaleMode::Original),
                computed_node,
            );

            let image = render_svg_image(svg_asset, target_size, tint_color);
            let rendered_size = UVec2::new(image.width(), image.height());
            let handle = images.add(image);

            commands.entity(entity).insert((
                ImageNode::new(handle),
                AppliedSvgState {
                    color: tint_color,
                    size: rendered_size,
                },
            ));
        }
    }
}

/// Updates the [`Sprite`] image when [`SvgColor`] changes on [`Svg2d`] entities.
fn update_svg_sprite_colors(
    mut commands: Commands,
    svg_assets: Res<Assets<SvgFile>>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(Entity, &Svg2d, &SvgColor, &mut Sprite, &AppliedSvgState), Changed<SvgColor>>,
) {
    for (entity, svg_component, color, mut sprite, applied_state) in &mut query {
        // Skip if color hasn't actually changed
        if color.0 == applied_state.color {
            continue;
        }

        if let Some(svg_asset) = svg_assets.get(&svg_component.0) {
            let image = if color.0 == Color::WHITE {
                svg_asset.image.clone()
            } else {
                svg_asset
                    .render_with_color(color.0, RenderAssetUsages::default())
                    .unwrap_or_else(|_| svg_asset.image.clone())
            };
            let size = UVec2::new(image.width(), image.height());
            let handle = images.add(image);
            sprite.image = handle;
            commands.entity(entity).insert(AppliedSvgState {
                color: color.0,
                size,
            });
        }
    }
}

/// Updates the [`ImageNode`] image when color, scale mode, or node size changes on [`SvgUi`] entities.
fn update_svg_ui_on_changes(
    mut commands: Commands,
    svg_assets: Res<Assets<SvgFile>>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<
        (
            Entity,
            &SvgUi,
            &SvgColor,
            &SvgScaleMode,
            Option<&ComputedNode>,
            &mut ImageNode,
            &AppliedSvgState,
        ),
        Or<(
            Changed<SvgColor>,
            Changed<SvgScaleMode>,
            Changed<ComputedNode>,
        )>,
    >,
) {
    for (entity, svg_component, color, scale_mode, computed_node, mut image_node, applied_state) in
        &mut query
    {
        let Some(svg_asset) = svg_assets.get(&svg_component.handle) else {
            continue;
        };

        // Determine target size based on scale mode
        let target_size = compute_target_size(svg_asset, scale_mode, computed_node);

        // Skip if nothing has actually changed
        if color.0 == applied_state.color && target_size == applied_state.size {
            continue;
        }

        let image = render_svg_image(svg_asset, target_size, color.0);
        let rendered_size = UVec2::new(image.width(), image.height());
        let handle = images.add(image);
        image_node.image = handle;

        commands.entity(entity).insert(AppliedSvgState {
            color: color.0,
            size: rendered_size,
        });
    }
}

/// Syncs [`SvgColor`] from [`TextColor`] for entities with [`SvgColorMode::InheritTextColor`].
/// Only checks the entity's own [`TextColor`] component; if not found, the original
/// [`SvgColor`] is preserved (no parent hierarchy traversal).
///
/// The sync is triggered when:
/// - A new `SvgColorMode` is added (initialization)
/// - The `SvgColorMode` changes to `InheritTextColor`
/// - The entity's own `TextColor` changes
fn sync_svg_color_from_text_color(
    mut query: Query<
        (&mut SvgColor, &SvgColorMode, Option<&TextColor>),
        Or<(
            Added<SvgColorMode>,
            Changed<SvgColorMode>,
            Changed<TextColor>,
        )>,
    >,
) {
    for (mut svg_color, color_mode, text_color) in &mut query {
        if *color_mode != SvgColorMode::InheritTextColor {
            continue;
        }

        // Only use TextColor if the entity itself has one
        if let Some(text_color) = text_color {
            if svg_color.0 != text_color.0 {
                svg_color.0 = text_color.0;
            }
        }
        // If no TextColor is present, keep the current SvgColor (original color)
    }
}

/// Computes the target rendering size based on scale mode and computed node size.
fn compute_target_size(
    svg_asset: &SvgFile,
    scale_mode: &SvgScaleMode,
    computed_node: Option<&ComputedNode>,
) -> UVec2 {
    let original_size = svg_asset.original_size();

    match scale_mode {
        SvgScaleMode::Original => original_size,
        SvgScaleMode::FitNode => {
            if let Some(node) = computed_node {
                let node_size = node.size();
                if node_size.x > 0.0 && node_size.y > 0.0 {
                    // Calculate scale to fit while maintaining aspect ratio
                    let scale_x = node_size.x / original_size.x as f32;
                    let scale_y = node_size.y / original_size.y as f32;
                    let scale = scale_x.min(scale_y);

                    UVec2::new(
                        ((original_size.x as f32 * scale).ceil() as u32).max(1),
                        ((original_size.y as f32 * scale).ceil() as u32).max(1),
                    )
                } else {
                    original_size
                }
            } else {
                original_size
            }
        }
        SvgScaleMode::Custom(scale) => UVec2::new(
            ((original_size.x as f32 * scale).ceil() as u32).max(1),
            ((original_size.y as f32 * scale).ceil() as u32).max(1),
        ),
    }
}

/// Renders an SVG image at the specified size with the given color.
fn render_svg_image(svg_asset: &SvgFile, target_size: UVec2, color: Color) -> Image {
    let original_size = svg_asset.original_size();

    if target_size == original_size && color == Color::WHITE {
        // Use pre-rendered image
        svg_asset.image.clone()
    } else if target_size == original_size {
        // Only color changed, use color-only render
        svg_asset
            .render_with_color(color, RenderAssetUsages::default())
            .unwrap_or_else(|_| svg_asset.image.clone())
    } else {
        // Size changed, use full re-render
        let color_opt = if color == Color::WHITE {
            None
        } else {
            Some(color)
        };
        svg_asset
            .render_at_size(target_size, color_opt, RenderAssetUsages::default())
            .unwrap_or_else(|_| svg_asset.image.clone())
    }
}
