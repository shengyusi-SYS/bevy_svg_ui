use crate::raster::plugin::{
    auto::AutoRaster,
    queue::{AppliedRaster, RasterIntent, RasterQueue},
};
use bevy::{prelude::*, tasks::futures::check_ready};

pub fn apply_finished_rasters(
    mut queue: ResMut<RasterQueue>,
    mut images: ResMut<Assets<Image>>,
    mut auto: Query<&mut AutoRaster>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
    mut nodes: Query<&mut ImageNode>,
    colors: Query<&crate::effects::components::SvgColor>,
    svg_assets: Res<Assets<crate::vector::asset::SvgAsset>>,
    ui_handles: Query<&crate::raster::component::UiSvg>,
    world_handles: Query<&crate::raster::component::Svg>,
) {
    let mut index = 0;
    while index < queue.0.len() {
        let Some(result) = check_ready(&mut queue.0[index].task) else {
            index += 1;
            continue;
        };
        let pending = queue.0.remove(index);

        let Some(image) = result else {
            debug!("Rasterisation cancelled for {:?}", pending.entity);
            continue;
        };

        // A worker owns its captured tree, not the entity's current binding.
        // Validate before registering an image or acknowledging this request.
        let current = ui_handles
            .get(pending.entity)
            .ok()
            .and_then(|svg| svg_assets.get(&svg.0))
            .or_else(|| {
                world_handles
                    .get(pending.entity)
                    .ok()
                    .and_then(|svg| svg_assets.get(&svg.0))
            });
        if current.is_none_or(|asset| !std::sync::Arc::ptr_eq(&asset.0, &pending.source)) {
            continue;
        }
        let handle = images.add(image);
        debug!(
            "APPLY {:?} {:?} -> {:?}",
            pending.entity, pending.intent, pending.size
        );
        let current_color = colors.get(pending.entity).map(|color| color.0);
        match pending.intent {
            RasterIntent::InsertSprite { color, world } => {
                commands.entity(pending.entity).try_insert(Sprite {
                    image: handle,
                    color: current_color.unwrap_or(color),
                    custom_size: Some(world),
                    ..default()
                });
            }
            RasterIntent::SwapSpriteImage { world } => {
                if let Ok(mut sprite) = sprites.get_mut(pending.entity) {
                    sprite.image = handle;
                    sprite.custom_size = Some(world);
                }
            }
            RasterIntent::InsertImageNode(color) => {
                commands.entity(pending.entity).try_insert(ImageNode {
                    image: handle,
                    color: current_color.unwrap_or(color),
                    ..default()
                });
            }
            RasterIntent::SwapImageNodeImage => {
                if let Ok(mut node) = nodes.get_mut(pending.entity) {
                    node.image = handle;
                }
            }
        }

        if ui_handles.contains(pending.entity) {
            commands.entity(pending.entity).try_insert(AppliedRaster {
                source: pending.source,
                size: pending.size,
            });
        }
        if let Ok(mut state) = auto.get_mut(pending.entity) {
            state.applied = pending.size;
        }
    }
}
