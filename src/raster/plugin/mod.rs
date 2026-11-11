use bevy::{
    camera::visibility::VisibilitySystems, prelude::*, transform::TransformSystems, ui::UiSystems,
};

mod apply;
mod auto;
mod color;
mod loading;
mod macros;
mod queue;
mod size;
mod sizing;

pub struct SvgRasterPlugin;

impl Plugin for SvgRasterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<bevy::window::RequestRedraw>()
            .init_resource::<queue::RasterQueue>()
            .add_systems(
                Update,
                (
                    color::sync_removed_svg_color,
                    color::sync_svg_color_changes,
                    loading::ui::handle_ui_svg_removed,
                    loading::world::handle_svg_loaded,
                    loading::world::handle_svg_modified,
                    loading::world::handle_svg_removed,
                    size::sync_removed_svg_size,
                    size::sync_svg_size_changes,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    (
                        queue::cancel_rasters_for_dead_entities,
                        apply::apply_finished_rasters,
                        auto::cleanup_auto_rasters,
                    )
                        .chain(),
                    auto::world::sync_svg_auto_sizes
                        .after(TransformSystems::Propagate)
                        .after(VisibilitySystems::CheckVisibility)
                        .after(apply::apply_finished_rasters),
                    redraw_while_pending
                        .after(auto::world::sync_svg_auto_sizes)
                        .after(auto::ui::sync_ui_svg_auto_sizes)
                        .after(apply::apply_finished_rasters),
                    auto::ui::sync_ui_svg_auto_sizes
                        .after(UiSystems::Layout)
                        .after(VisibilitySystems::CheckVisibility)
                        .after(apply::apply_finished_rasters),
                ),
            )
            .add_observer(queue::cancel_raster_on_despawn_svg)
            .add_observer(queue::cancel_raster_on_despawn_ui_svg);
    }
}

// Raster workers do not emit AssetServer completion events. Keep reactive hosts
// awake only while they have pending images to install.
fn redraw_while_pending(
    queue: Res<queue::RasterQueue>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    if !queue.0.is_empty() {
        redraw.write(bevy::window::RequestRedraw);
    }
}
