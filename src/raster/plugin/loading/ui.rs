use crate::{
    raster::{
        component::UiSvg,
        plugin::{macros::read_events, queue::RasterQueue},
    },
    vector::asset::SvgAsset,
};
use bevy::prelude::*;

pub fn handle_ui_svg_removed(
    mut commands: Commands,
    mut queue: ResMut<RasterQueue>,
    mut svg_events: MessageReader<AssetEvent<SvgAsset>>,
    query: Query<(Entity, &UiSvg), With<ImageNode>>,
) {
    let removed_ids = read_events!(svg_events, AssetEvent::Removed | AssetEvent::Unused);

    for (entity, svg) in query {
        let id = svg.0.id();
        if removed_ids.contains(&id) {
            queue.cancel(entity);
            commands.entity(entity).remove::<ImageNode>();
            info!("Removed `ImageNode` for `{id}` from entity {entity:?}");
        }
    }
}
