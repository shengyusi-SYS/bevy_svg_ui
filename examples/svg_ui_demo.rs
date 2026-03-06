//! SVG UI Demo
//!
//! This example demonstrates the usage of `bevy_svg_ui` crate for rendering SVG files
//! as UI elements with comprehensive layout, color, and supersampling features.
//!
//! ## Features Demonstrated:
//! - **Flex Layout**: Row and column arrangements with gaps, alignment, and padding
//! - **Border Styling**: Colored borders with border radius for visual separation
//! - **SvgColor**: Runtime color modification and tinting
//! - **SvgColorMode::InheritTextColor**: Inherit color from TextColor component
//! - **SvgScaleMode**: Supersampling anti-aliasing (1x, 2x, 4x) for sharper rendering
//!
//! ## Anti-Aliasing Analysis:
//! The SVG icons are stroke-based (using stroke-width="2"). When rendered at small sizes,
//! the 2px stroke gets scaled down, causing aliasing on diagonal edges. The supersampling
//! modes (2x, 4x) render at higher resolution and then downscale, which provides better
//! anti-aliasing through the texture filtering when displayed.

use bevy::prelude::*;
use bevy_svg_ui::prelude::*;

/// Helper macro to spawn a scale comparison demo box
macro_rules! spawn_scale_demo {
    ($parent:expr, $icon:expr, $scale_mode:expr, $label:expr, $accent_color:expr) => {
        $parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                BorderColor::all($accent_color.with_alpha(0.7)),
            ))
            .with_children(|col| {
                col.spawn((
                    SvgUi::new($icon),
                    $scale_mode,
                    SvgColor::new($accent_color),
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        ..default()
                    },
                ));

                col.spawn((
                    Text::new($label),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor($accent_color),
                ));
            });
    };
}

/// Helper macro to spawn a size comparison box
macro_rules! spawn_size_box {
    ($parent:expr, $icon:expr, $size:expr, $scale_mode:expr, $color:expr) => {
        $parent
            .spawn((Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },))
            .with_children(|col| {
                col.spawn((
                    Node {
                        width: Val::Px($size),
                        height: Val::Px($size),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                ))
                .with_children(|box_node| {
                    box_node.spawn((
                        SvgUi::new($icon),
                        $scale_mode,
                        SvgColor::new($color),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                });

                col.spawn((
                    Text::new(format!("{}px", $size as i32)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.7)),
                ));
            });
    };
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SVG UI Demo - Flex Layout & Supersampling".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SvgPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_colors)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Try to create symlink from assets/icons to crates/frontend/assets/icons
    // Use CARGO_MANIFEST_DIR to get the workspace root
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let workspace_root = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .expect("Failed to find workspace root");

        let target = workspace_root.join("crates/frontend/assets/icons");
        let link = workspace_root.join("crates/bevy_svg_ui/assets/icons");

        if !link.exists() {
            // Ensure parent directory exists
            if let Some(parent) = link.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            #[cfg(target_os = "windows")]
            let result = std::os::windows::fs::symlink_dir(&target, &link);
            
            #[cfg(unix)]
            let result = std::os::unix::fs::symlink(&target, &link);
            
            if let Err(e) = result {
                eprintln!(
                    "Failed to create symlink: {}. This is non-fatal, continuing...",
                    e
                );
            } else {
                println!("Created symlink: {:?} -> {:?}", link, target);
            }
        }
    }

    // Spawn camera
    commands.spawn(Camera2d::default());

    // Load various SVG icons
    let icon_library: Handle<SvgFile> = asset_server.load("icons/library-big.svg");
    let icon_play: Handle<SvgFile> = asset_server.load("icons/play.svg");
    let icon_headphones: Handle<SvgFile> = asset_server.load("icons/headphones.svg");
    let icon_list_music: Handle<SvgFile> = asset_server.load("icons/list-music.svg");
    let icon_speaker: Handle<SvgFile> = asset_server.load("icons/speaker.svg");
    let icon_volume: Handle<SvgFile> = asset_server.load("icons/volume-2.svg");
    let icon_shuffle: Handle<SvgFile> = asset_server.load("icons/shuffle.svg");
    let icon_repeat: Handle<SvgFile> = asset_server.load("icons/repeat.svg");
    let icon_skip_back: Handle<SvgFile> = asset_server.load("icons/skip-back.svg");
    let icon_skip_forward: Handle<SvgFile> = asset_server.load("icons/skip-forward.svg");
    let icon_chevron: Handle<SvgFile> = asset_server.load("icons/chevron-right.svg");
    let icon_bolt: Handle<SvgFile> = asset_server.load("icons/bolt.svg");

    // === Root Container: Full screen with dark background ===
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(16.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.10)),
        ))
        .with_children(|root| {
            // === Header Section ===
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 0.95)),
                BorderColor::all(Color::srgb(0.4, 0.6, 1.0)),
            ))
            .with_children(|header| {
                header.spawn((
                    Text::new("SVG UI Demo - Flex Layout, Colors & Supersampling"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 1.0)),
                ));
            });

            // === Main Content: Three column layout ===
            root.spawn((Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                ..default()
            },))
            .with_children(|main_content| {
                // === Left Panel: Color Variations ===
                main_content
                    .spawn((
                        Node {
                            width: Val::Percent(33.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(12.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.10, 0.15, 0.95)),
                        BorderColor::all(Color::srgb(0.8, 0.3, 0.5)),
                    ))
                    .with_children(|left_panel| {
                        // Section Title
                        left_panel.spawn((
                            Text::new("Color Tinting"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.3, 0.5)),
                        ));

                        // Color Grid - 4x2 layout
                        left_panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::Wrap,
                                    column_gap: Val::Px(8.0),
                                    row_gap: Val::Px(8.0),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                            ))
                            .with_children(|grid| {
                                let colors = [
                                    Color::WHITE,
                                    Color::srgb(1.0, 0.3, 0.3),
                                    Color::srgb(0.3, 1.0, 0.3),
                                    Color::srgb(0.3, 0.5, 1.0),
                                    Color::srgb(1.0, 1.0, 0.3),
                                    Color::srgb(1.0, 0.5, 0.0),
                                    Color::srgb(0.8, 0.3, 1.0),
                                    Color::srgb(0.3, 1.0, 1.0),
                                ];

                                for color in colors {
                                    grid.spawn((
                                        Node {
                                            padding: UiRect::all(Val::Px(6.0)),
                                            border: UiRect::all(Val::Px(2.0)),
                                            border_radius: BorderRadius::all(Val::Px(6.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.8)),
                                        BorderColor::all(color.with_alpha(0.5)),
                                    ))
                                    .with_children(|icon_box| {
                                        icon_box.spawn((
                                            SvgUi::new(icon_library.clone()),
                                            SvgColor::new(color),
                                            SvgScaleMode::Custom(2.0),
                                            Node {
                                                width: Val::Px(32.0),
                                                height: Val::Px(32.0),
                                                ..default()
                                            },
                                        ));
                                    });
                                }
                            });

                        // Animated Color Section
                        left_panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
                                BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                            ))
                            .with_children(|anim_row| {
                                anim_row.spawn((
                                    SvgUi::new(icon_headphones.clone()),
                                    SvgColorMode::InheritTextColor,
                                    TextColor(Color::srgb(1.0, 0.8, 0.0)),
                                    SvgScaleMode::Custom(2.0),
                                    Node {
                                        width: Val::Px(40.0),
                                        height: Val::Px(40.0),
                                        ..default()
                                    },
                                    AnimatedColor,
                                ));

                                anim_row.spawn((
                                    Text::new("Animated\n(InheritTextColor)"),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.7, 0.7, 0.8)),
                                ));
                            });

                        // Multiple Icons Row
                        left_panel.spawn((
                            Text::new("Icon Variety"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.7)),
                        ));

                        left_panel
                            .spawn((Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(8.0),
                                row_gap: Val::Px(8.0),
                                ..default()
                            },))
                            .with_children(|icons_row| {
                                let icons = [
                                    (icon_play.clone(), Color::srgb(0.4, 0.9, 0.5)),
                                    (icon_shuffle.clone(), Color::srgb(0.9, 0.6, 0.3)),
                                    (icon_repeat.clone(), Color::srgb(0.5, 0.7, 1.0)),
                                    (icon_volume.clone(), Color::srgb(0.9, 0.4, 0.6)),
                                    (icon_chevron.clone(), Color::srgb(0.6, 0.9, 0.8)),
                                    (icon_bolt.clone(), Color::srgb(1.0, 0.9, 0.3)),
                                ];

                                for (icon, color) in icons {
                                    icons_row.spawn((
                                        SvgUi::new(icon),
                                        SvgColor::new(color),
                                        SvgScaleMode::Custom(2.0),
                                        Node {
                                            width: Val::Px(28.0),
                                            height: Val::Px(28.0),
                                            ..default()
                                        },
                                    ));
                                }
                            });
                    });

                // === Center Panel: Supersampling Comparison ===
                main_content
                    .spawn((
                        Node {
                            width: Val::Percent(33.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(12.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.10, 0.15, 0.95)),
                        BorderColor::all(Color::srgb(0.3, 0.7, 0.4)),
                    ))
                    .with_children(|center_panel| {
                        // Section Title
                        center_panel.spawn((
                            Text::new("Supersampling AA"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.9, 0.5)),
                        ));

                        // Supersampling comparison: 1x vs 2x vs 4x
                        center_panel
                            .spawn((Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceEvenly,
                                column_gap: Val::Px(8.0),
                                ..default()
                            },))
                            .with_children(|comparison| {
                                spawn_scale_demo!(
                                    comparison,
                                    icon_play.clone(),
                                    SvgScaleMode::Original,
                                    "1x",
                                    Color::srgb(1.0, 0.4, 0.4)
                                );

                                spawn_scale_demo!(
                                    comparison,
                                    icon_play.clone(),
                                    SvgScaleMode::Custom(2.0),
                                    "2x",
                                    Color::srgb(0.4, 1.0, 0.4)
                                );

                                spawn_scale_demo!(
                                    comparison,
                                    icon_play.clone(),
                                    SvgScaleMode::Custom(4.0),
                                    "4x",
                                    Color::srgb(0.4, 0.7, 1.0)
                                );
                            });

                        // Analysis Note
                        center_panel.spawn((
                            Text::new("Note: SVG icons use stroke-width=2.\nHigher supersample renders at\nlarger size, then downscales."),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.6)),
                        ));

                        // FitNode Demo Title
                        center_panel.spawn((
                            Text::new("FitNode Scale Mode"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.8)),
                        ));

                        // Size comparison with FitNode
                        center_panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::End,
                                    column_gap: Val::Px(12.0),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                            ))
                            .with_children(|fit_row| {
                                let sizes = [20.0_f32, 36.0, 56.0, 80.0];
                                for size in sizes {
                                    spawn_size_box!(
                                        fit_row,
                                        icon_speaker.clone(),
                                        size,
                                        SvgScaleMode::FitNode,
                                        Color::srgb(0.9, 0.7, 0.3)
                                    );
                                }
                            });

                        // Additional comparison: Same size, different icons
                        center_panel.spawn((
                            Text::new("Diagonal Edge Analysis"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.8)),
                        ));

                        center_panel
                            .spawn((Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(16.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },))
                            .with_children(|diag_row| {
                                // Chevron has strong diagonal - good for AA testing
                                for scale in [1.0_f32, 2.0, 4.0] {
                                    diag_row
                                        .spawn((Node {
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            row_gap: Val::Px(4.0),
                                            ..default()
                                        },))
                                        .with_children(|col| {
                                            col.spawn((
                                                SvgUi::new(icon_chevron.clone()),
                                                if scale == 1.0 {
                                                    SvgScaleMode::Original
                                                } else {
                                                    SvgScaleMode::Custom(scale)
                                                },
                                                SvgColor::new(Color::WHITE),
                                                Node {
                                                    width: Val::Px(48.0),
                                                    height: Val::Px(48.0),
                                                    ..default()
                                                },
                                            ));

                                            col.spawn((
                                                Text::new(format!("{}x", scale as i32)),
                                                TextFont {
                                                    font_size: 11.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.6)),
                                            ));
                                        });
                                }
                            });
                    });

                // === Right Panel: Flex Layout Demo ===
                main_content
                    .spawn((
                        Node {
                            width: Val::Percent(33.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(12.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.10, 0.15, 0.95)),
                        BorderColor::all(Color::srgb(0.5, 0.5, 0.9)),
                    ))
                    .with_children(|right_panel| {
                        // Section Title
                        right_panel.spawn((
                            Text::new("Flex Layout"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 1.0)),
                        ));

                        // Row layout example
                        right_panel.spawn((
                            Text::new("Row with SpaceBetween"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.6)),
                        ));

                        right_panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                            ))
                            .with_children(|row| {
                                for icon in [
                                    icon_skip_back.clone(),
                                    icon_play.clone(),
                                    icon_skip_forward.clone(),
                                ] {
                                    row.spawn((
                                        SvgUi::new(icon),
                                        SvgColor::new(Color::WHITE),
                                        SvgScaleMode::Custom(2.0),
                                        Node {
                                            width: Val::Px(24.0),
                                            height: Val::Px(24.0),
                                            ..default()
                                        },
                                    ));
                                }
                            });

                        // Column layout example
                        right_panel.spawn((
                            Text::new("Column with gap"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.6)),
                        ));

                        right_panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(6.0),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                            ))
                            .with_children(|col| {
                                let items = [
                                    (icon_list_music.clone(), "Playlist", Color::srgb(0.4, 0.8, 1.0)),
                                    (icon_library.clone(), "Library", Color::srgb(0.9, 0.5, 0.3)),
                                    (icon_speaker.clone(), "Speakers", Color::srgb(0.5, 0.9, 0.5)),
                                ];

                                for (icon, label, color) in items {
                                    col.spawn((Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(8.0),
                                        ..default()
                                    },))
                                    .with_children(|item_row| {
                                        item_row.spawn((
                                            SvgUi::new(icon),
                                            SvgColor::new(color),
                                            SvgScaleMode::Custom(2.0),
                                            Node {
                                                width: Val::Px(20.0),
                                                height: Val::Px(20.0),
                                                ..default()
                                            },
                                        ));

                                        item_row.spawn((
                                            Text::new(label),
                                            TextFont {
                                                font_size: 13.0,
                                                ..default()
                                            },
                                            TextColor(color),
                                        ));
                                    });
                                }
                            });

                        // Nested flex example
                        right_panel.spawn((
                            Text::new("Nested Flex (Card)"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.6)),
                        ));

                        right_panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    padding: UiRect::all(Val::Px(10.0)),
                                    column_gap: Val::Px(10.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
                                BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                            ))
                            .with_children(|card| {
                                // Icon
                                card.spawn((
                                    Node {
                                        padding: UiRect::all(Val::Px(8.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.3, 0.5, 0.8, 0.3)),
                                ))
                                .with_children(|icon_bg| {
                                    icon_bg.spawn((
                                        SvgUi::new(icon_headphones.clone()),
                                        SvgColor::new(Color::srgb(0.5, 0.8, 1.0)),
                                        SvgScaleMode::Custom(2.0),
                                        Node {
                                            width: Val::Px(28.0),
                                            height: Val::Px(28.0),
                                            ..default()
                                        },
                                    ));
                                });

                                // Text content
                                card.spawn((Node {
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },))
                                .with_children(|text_col| {
                                    text_col.spawn((
                                        Text::new("Now Playing"),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                                    ));

                                    text_col.spawn((
                                        Text::new("Flex Layout Demo"),
                                        TextFont {
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.5, 0.5, 0.6)),
                                    ));
                                });
                            });
                    });
            });

            // === Bottom Panel: Player Controls ===
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
                BorderColor::all(Color::srgb(0.35, 0.35, 0.45)),
            ))
            .with_children(|controls| {
                // Left: Info
                controls
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },))
                    .with_children(|left| {
                        left.spawn((
                            SvgUi::new(icon_list_music.clone()),
                            SvgColor::new(Color::srgb(0.5, 0.7, 1.0)),
                            SvgScaleMode::Custom(2.0),
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                ..default()
                            },
                        ));

                        left.spawn((
                            Text::new("bevy_svg_ui Demo"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.8)),
                        ));
                    });

                // Center: Playback Controls
                controls
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.8)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                    ))
                    .with_children(|center| {
                        center.spawn((
                            SvgUi::new(icon_shuffle.clone()),
                            SvgColor::new(Color::srgb(0.4, 0.4, 0.5)),
                            SvgScaleMode::Custom(2.0),
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                        ));

                        center.spawn((
                            SvgUi::new(icon_skip_back.clone()),
                            SvgColor::new(Color::WHITE),
                            SvgScaleMode::Custom(2.0),
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                        ));

                        // Play Button
                        center
                            .spawn((
                                Node {
                                    padding: UiRect::all(Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(50.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.25, 0.6, 0.35)),
                                BorderColor::all(Color::srgb(0.35, 0.8, 0.45)),
                            ))
                            .with_children(|play_btn| {
                                play_btn.spawn((
                                    SvgUi::new(icon_play.clone()),
                                    SvgColor::new(Color::WHITE),
                                    SvgScaleMode::Custom(2.0),
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                ));
                            });

                        center.spawn((
                            SvgUi::new(icon_skip_forward.clone()),
                            SvgColor::new(Color::WHITE),
                            SvgScaleMode::Custom(2.0),
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                        ));

                        center.spawn((
                            SvgUi::new(icon_repeat.clone()),
                            SvgColor::new(Color::srgb(0.4, 0.4, 0.5)),
                            SvgScaleMode::Custom(2.0),
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                        ));
                    });

                // Right: Volume
                controls
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },))
                    .with_children(|right| {
                        right.spawn((
                            SvgUi::new(icon_volume.clone()),
                            SvgColor::new(Color::srgb(0.6, 0.6, 0.7)),
                            SvgScaleMode::Custom(2.0),
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                        ));

                        // Volume bar
                        right
                            .spawn((
                                Node {
                                    width: Val::Px(80.0),
                                    height: Val::Px(4.0),
                                    border_radius: BorderRadius::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: Val::Percent(65.0),
                                        height: Val::Percent(100.0),
                                        border_radius: BorderRadius::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.35, 0.6, 0.9)),
                                ));
                            });
                    });
            });
        });
}

/// Marker component for animated color demonstration
#[derive(Component)]
struct AnimatedColor;

/// Animates the TextColor of entities with AnimatedColor marker
fn animate_colors(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedColor>>) {
    let t = time.elapsed_secs();
    let hue = (t * 0.5) % 1.0;

    let (r, g, b) = hsl_to_rgb(hue, 0.9, 0.6);

    for mut text_color in &mut query {
        text_color.0 = Color::srgb(r, g, b);
    }
}

/// Simple HSL to RGB conversion
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}
