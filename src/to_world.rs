//! An optional component for converting positions between "terminal space"
//! and world space.

use bevy::{
    math::{IVec2, Mat4, UVec2, Vec2, Vec3},
    prelude::{
        App, Assets, Camera, Changed, Component, Entity, GlobalTransform, Image, Or, Plugin, Query,
        Res, Update, With,
    },
    render::camera::{ManualTextureViews, RenderTarget},
    window::{PrimaryWindow, Window, WindowRef},
};
use sark_grids::GridPoint;

use crate::{
    renderer::{TerminalLayout, TileScaling},
    Terminal,
};

pub(crate) struct ToWorldPlugin;

impl Plugin for ToWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_from_terminal, update_from_camera));
    }
}

/// A component for converting positions between World Space and
/// "Terminal Space".
///
/// When you add this to a terminal it will track the various properties of the
/// terminal and camera, and provide functions for converting positions.
#[derive(Default, Component)]
pub struct ToWorld {
    term_size: UVec2,
    term_pos: Vec3,
    layout: TerminalLayout,
    camera_entity: Option<Entity>,
    ndc_to_world: Mat4,
    camera_pos: Vec3,
    viewport_pos: Vec2,
    viewport_size: Option<Vec2>,
}

impl ToWorld {
    /// Convert a tile position (bottom left corner) to it's corresponding
    /// world position.
    pub fn tile_to_world(&self, tile: impl GridPoint) -> Vec3 {
        let term_pos = self.term_pos.truncate();
        let term_offset = self.term_size.as_vec2() * Vec2::from(self.layout.pivot);
        (tile.as_vec2() + term_pos - term_offset).extend(self.term_pos.z)
    }

    /// Convert a tile center to it's corresponding world position.
    pub fn tile_center_to_world(&self, tile: impl GridPoint) -> Vec3 {
        let center_offset = (self.world_unit() / 2.0).extend(0.0);
        self.tile_to_world(tile) + center_offset
    }

    pub fn world_to_tile(&self, world: Vec2) -> IVec2 {
        let term_pos = self.term_pos.truncate();
        let term_offset = self.term_size.as_vec2() * Vec2::from(self.layout.pivot);
        let xy = world - term_pos + term_offset;
        xy.floor().as_ivec2()
    }

    /// The size of a single world unit, accounting for `TileScaling`.
    pub fn world_unit(&self) -> Vec2 {
        match self.layout.scaling {
            TileScaling::World => Vec2::ONE,
            TileScaling::Pixels => self.layout.pixels_per_tile.as_vec2(),
        }
    }

    /// Convert a position from screen space (ie: Cursor position) to world space.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Option<Vec2> {
        if let Some(viewport_size) = self.viewport_size {
            let screen_pos = screen_pos - self.viewport_pos;
            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / viewport_size) * 2.0 - Vec2::ONE;

            // use it to convert ndc to world-space coordinates
            let world_pos = self.ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            Some(world_pos.truncate())
        } else {
            None
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_from_terminal(
    mut q_term: Query<
        (&mut ToWorld, &Terminal, &GlobalTransform, &TerminalLayout),
        Or<(Changed<Terminal>, Changed<TerminalLayout>)>,
    >,
) {
    for (mut to_world, term, transform, layout) in q_term.iter_mut() {
        to_world.term_size = term.size();
        to_world.layout = layout.clone();
        to_world.term_pos = transform.translation();
    }
}

#[allow(clippy::type_complexity)]
fn update_from_camera(
    q_cam: Query<
        (Entity, &Camera, &GlobalTransform),
        Or<(Changed<Camera>, Changed<GlobalTransform>)>,
    >,
    mut q_to_world: Query<&mut ToWorld>,
    windows: Query<&Window>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    manual_texture_views: Res<ManualTextureViews>,
) {
    if q_cam.is_empty() {
        return;
    }

    for mut tw in q_to_world.iter_mut() {
        // If no camera is explicitly set, choose the first camera we can find
        if tw.camera_entity.is_none() {
            tw.camera_entity = Some(q_cam.iter().next().unwrap().0);
        }

        for (cam_entity, cam, t) in q_cam.iter() {
            if cam_entity != tw.camera_entity.unwrap() {
                continue;
            }

            tw.camera_pos = t.translation();
            tw.ndc_to_world = t.compute_matrix() * cam.clip_from_view().inverse();

            if let Some(vp) = &cam.viewport {
                tw.viewport_pos = vp.physical_position.as_vec2();
                tw.viewport_size = Some(vp.physical_size.as_vec2());
            } else {
                tw.viewport_pos = Vec2::ZERO;
                let res = match &cam.target {
                    RenderTarget::Window(win_ref) => {
                        let window = match win_ref {
                            WindowRef::Primary => primary_window.get_single().ok(),
                            WindowRef::Entity(win_entity) => windows.get(*win_entity).ok(),
                        };

                        window.map(|window| Vec2::new(window.width(), window.height()))
                        // if let Some(window) = windows.get(*win_id) {
                        //     Some(Vec2::new(window.width(), window.height()))
                        // } else {
                        //     None
                        // }
                    }
                    RenderTarget::Image(image) => {
                        images.get(image).map(|image| image.size().as_vec2())
                        // if let Some(image) = images.get(image) {
                        //     Some(image.size())
                        // } else {
                        //     None
                        // }
                    }
                    RenderTarget::TextureView(texture_view) => manual_texture_views
                        .get(texture_view)
                        .map(|manual_texture_view| manual_texture_view.size.as_vec2()),
                };
                tw.viewport_size = res;
            }
        }
    }
}
