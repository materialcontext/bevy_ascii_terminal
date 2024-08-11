use bevy::prelude::{default, Bundle, Changed, Component, Query, With};
use sark_grids::{GridPoint, Pivot, Size2d};

use crate::{
    renderer::{self, TileScaling},
    Border, Terminal, TerminalFont, TerminalLayout,
};

/// A bundle with all the required components for a terminal.
///
/// Can specify some properties of the terminal on initilaization.
#[derive(Bundle, Default)]
pub struct TerminalBundle {
    pub terminal: Terminal,
    pub renderer: renderer::TerminalRenderBundle,
    pub layout: TerminalLayout,
    pub font: TerminalFont,
}

impl From<Terminal> for TerminalBundle {
    fn from(terminal: Terminal) -> Self {
        let layout = TerminalLayout::from(&terminal);
        TerminalBundle {
            terminal,
            layout,
            ..default()
        }
    }
}

impl TerminalBundle {
    pub fn new() -> Self {
        TerminalBundle::default()
    }

    /// Set the initial size of the terminal.
    pub fn with_size(mut self, size: impl Size2d) -> Self {
        self.terminal.resize(size.as_array());
        self.layout.set_size(size.as_ivec2());
        self
    }

    pub fn with_border(mut self, border: Border) -> Self {
        self.terminal.set_border(border.clone());
        self.layout.set_border(Some(border));
        self
    }

    pub fn with_pivot(mut self, pivot: Pivot) -> Self {
        self.layout.pivot = pivot;
        self
    }

    pub fn with_font(mut self, font: TerminalFont) -> Self {
        self.font = font;
        self
    }

    pub fn with_position(mut self, pos: impl GridPoint) -> Self {
        let p = self.renderer.render_bundle.transform.translation;
        self.renderer.render_bundle.transform.translation = pos.as_vec2().extend(p.z);
        self.layout.pos = pos.as_ivec2();
        self
    }

    /// Sets the intial z position for the terminal.
    pub fn with_depth(mut self, depth: i32) -> Self {
        self.renderer.render_bundle.transform.translation.z = depth as f32;
        self
    }

    /// Sets the [TileScaling] for the terminal.
    pub fn with_tile_scaling(mut self, scaling: TileScaling) -> Self {
        self.layout.scaling = scaling;
        self
    }
}

/// If this component is added to a terminal the terminal will automatically be
/// cleared after every render.
#[derive(Default, Debug, Component)]
pub struct ClearAfterRender;

pub(crate) fn clear_after_render(
    mut q_term: Query<&mut Terminal, (Changed<Terminal>, With<ClearAfterRender>)>,
) {
    q_term.iter_mut().for_each(|mut t| t.clear());
}

