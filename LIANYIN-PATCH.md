# PlayMusic SVG integration

Based on upstream bevy_resvg 3.0.1, commit `72df44e`. The SVG parser and
rasterizer remain upstream resvg; this branch corrects the Bevy UI handoff.

- A single post-layout UI system owns initial rendering, sizing and content
  changes. It uses inherited visibility, physical ComputedNode dimensions and
  the remaining UiGlobalTransform scale; retained hidden UI keeps its raster.
- Requests and installed rasters carry their captured immutable tree identity.
  Same-size handle changes and content updates supersede earlier workers;
  installation validates the current tree before publishing the image.
- Rasterize the complete tree with `resvg::render`. Iterating root nodes and
  requiring a non-empty layer bounding box drops horizontal/vertical strokes.
- UI icons render at 2x on the worker, then resolve with tiny-skia bilinear
  filtering to the requested physical size. This adds no persistent GPU pixels
  or intrinsic layout feedback; SVG images use an explicit linear sampler.
- Use tiny-skia's `take_demultiplied` and an sRGB Image for Bevy's straight-alpha
  ImageNode path. Tint remains independent of the pixels.
- On async installation, read the entity's current tint instead of restoring
  the color captured when the worker began. Equal color checks do not write.
- UiSvg requires Node, SvgColor and automatic SvgSize. Pending raster work
  requests redraws so reactive hosts can install completed images.

PlayMusic owns monochrome currentColor loading, text/theme tint inheritance,
its icon registry and preservation of multicolor logos. Those policies are
not part of this dependency.

Integration regressions live in PlayMusic's
`crates/frontend/src/plugin/svg_icon_raster_tests.rs`. From that repository:

```sh
cargo xtask nextest run -p lianyin-frontend --lib --features gles,ui-debug,bevy/dynamic_linking --locked --offline -E 'test(plugin::svg_icon::)'
```

Remove these patches when upstream provides equivalent behavior. SVG changes
can still be hot-reloaded through Bevy's asset watcher; ordinary tint changes
only update ImageNode color and do not replace image handles.
