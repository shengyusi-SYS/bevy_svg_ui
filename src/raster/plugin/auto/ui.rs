//! One owner for initial UI rendering, resizing and SVG content/binding changes.
use crate::{
    effects::components::{SvgColor, SvgSize},
    raster::{
        component::UiSvg,
        plugin::{
            queue::{AppliedRaster, RasterIntent, RasterQueue},
            sizing::{ONE, is_auto, known_size, raster_size, round_f32},
        },
    },
    vector::asset::{IntSize, SvgAsset},
};
use bevy::prelude::*;

pub fn sync_ui_svg_auto_sizes(
    assets: Res<Assets<SvgAsset>>,
    mut queue: ResMut<RasterQueue>,
    nodes: Query<(
        Entity,
        &UiSvg,
        Option<&SvgSize>,
        &SvgColor,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        Option<&ImageNode>,
        Option<&AppliedRaster>,
    )>,
) {
    for (entity, svg, size, color, computed, transform, visibility, image, applied) in &nodes {
        let Some(vector) = assets.get(&svg.0) else {
            if queue.is_busy(entity) {
                queue.cancel(entity);
            }
            continue;
        };
        if !visibility.get() {
            continue;
        }
        let size = size.copied().unwrap_or_else(SvgSize::auto);
        let mut target = raster_size(vector, Some(&size));
        // Layout already includes DPI/UiScale. Only UI-transform magnification
        // remains. Zero geometry needs an intrinsic first image for measurement.
        let pixels = computed.size
            * Vec2::new(
                transform.matrix2.x_axis.length(),
                transform.matrix2.y_axis.length(),
            );
        if is_auto(&size) && pixels.is_finite() && pixels.min_element() >= 1.0 {
            let intrinsic = vector.0.size();
            let [width, height] = pixels
                .to_array()
                .map(|n| round_f32(n.ceil()).unwrap_or(ONE));
            target = IntSize {
                width: known_size(size.width, intrinsic.width(), width),
                height: known_size(size.height, intrinsic.height(), height),
            };
        }
        if image.is_some() && applied.is_some_and(|a| a.matches(vector, target)) {
            continue;
        }
        if queue.matches(entity, vector, target) {
            continue;
        }
        let intent = if image.is_some() {
            RasterIntent::SwapImageNodeImage
        } else {
            RasterIntent::InsertImageNode(color.0)
        };
        queue.request(entity, vector, target, intent);
    }
}
