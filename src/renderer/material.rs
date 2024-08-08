//! The material used for terminal rendering.
//!
//! By default the terminal expects a [code page 437](https://dwarffortresswiki.org/Tileset_repository)
//! texture with 16x16 characters. New font textures can be added to the assets directory and loaded via
//! the bevy `AssetLoader`.

use bevy::{
    math::Vec4,
    prelude::{
        default, Asset, Assets, Changed, Color, Handle, Image, Mesh, Or, Plugin, Query, Res,
        Shader, Vec2,
    },
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupShaderType, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
        texture::GpuImage,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use crate::{TerminalFont, TerminalLayout};

use super::{
    font::TerminalFontPlugin,
    mesh_data::{ATTRIBUTE_COLOR_BG, ATTRIBUTE_COLOR_FG, ATTRIBUTE_UV},
    //mesh::{ATTRIBUTE_COLOR_BG, ATTRIBUTE_COLOR_FG, ATTRIBUTE_UV},
    BuiltInFontHandles,
    TileScaling,
};

/// The default shader handle used by terminals.
pub const TERMINAL_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(3142086811234592509);

pub struct TerminalMaterialPlugin;

impl Plugin for TerminalMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            TerminalFontPlugin,
            Material2dPlugin::<TerminalMaterial>::default(),
        ));

        let mut shaders = app.world_mut().get_resource_mut::<Assets<Shader>>().expect(
            "Error initializing TerminalPlugin. Ensure TerminalPlugin is added AFTER
            DefaultPlugins during app initialization. (issue #1255)",
        );

        shaders.insert(
            &TERMINAL_MATERIAL_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("terminal.wgsl"), "terminal.wgsl"),
        );

        let fonts = app
            .world_mut()
            .get_resource::<BuiltInFontHandles>()
            .expect("Couldn't get font handles");
        let font = fonts.get(&TerminalFont::default());
        let material = TerminalMaterial::from(font.clone());

        app.world_mut()
            .resource_mut::<Assets<TerminalMaterial>>()
            .insert(&Handle::<TerminalMaterial>::default(), material);
    }
}

#[derive(AsBindGroup, Asset, Debug, Clone, TypePath)]
#[uniform(0, TerminalMaterialUniform)]
pub struct TerminalMaterial {
    /// This determines the "background color" for the texture,
    /// which will be clipped and replaced with a tile color.
    pub bg_clip_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Default for TerminalMaterial {
    fn default() -> Self {
        Self {
            bg_clip_color: Color::BLACK,
            texture: None,
        }
    }
}

impl From<Handle<Image>> for TerminalMaterial {
    fn from(image: Handle<Image>) -> Self {
        TerminalMaterial {
            texture: Some(image),
            ..default()
        }
    }
}

// NOTE: These must match the bit flags in shader.wgsl!
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct TerminalMaterialFlags: u32 {
        const TEXTURE           = (1 << 0);
        const NONE              = 0;
        const UNINITIALIZED     = 0xFFFF;
    }
}

/// The GPU representation of the uniform data of a [`TerminalMaterial`].
#[derive(Clone, Default, ShaderType)]
struct TerminalMaterialUniform {
    pub color: Vec4,
    pub flags: u32,
}

impl TerminalMaterialUniform {
    fn from_color(color: Color, flags: u32) -> TerminalMaterialUniform {
        let linear = color.to_linear();
        TerminalMaterialUniform {
            color: Vec4::from_array([linear.red, linear.green, linear.blue, linear.alpha]),
            flags,
        }
    }
}

impl AsBindGroupShaderType<TerminalMaterialUniform> for TerminalMaterial {
    fn as_bind_group_shader_type(&self, _: &RenderAssets<GpuImage>) -> TerminalMaterialUniform {
        let mut flags = TerminalMaterialFlags::NONE;
        if self.texture.is_some() {
            flags |= TerminalMaterialFlags::TEXTURE;
        }

        TerminalMaterialUniform::from_color(self.bg_clip_color, flags.bits())
    }
}

impl Material2d for TerminalMaterial {
    fn fragment_shader() -> ShaderRef {
        TERMINAL_MATERIAL_SHADER_HANDLE.into()
    }

    fn vertex_shader() -> ShaderRef {
        TERMINAL_MATERIAL_SHADER_HANDLE.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_UV.at_shader_location(1),
            ATTRIBUTE_COLOR_BG.at_shader_location(2),
            ATTRIBUTE_COLOR_FG.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn material_change(
    materials: Res<Assets<TerminalMaterial>>,
    images: Res<Assets<Image>>,
    mut q_term: Query<
        (&Handle<TerminalMaterial>, &mut TerminalLayout),
        Or<(Changed<Handle<TerminalMaterial>>, Changed<TerminalFont>)>,
    >,
) {
    for (handle, mut layout) in &mut q_term {
        if let Some(material) = materials.get(handle) {
            if let Some(image) = material.texture.clone() {
                if let Some(image) = images.get(&image) {
                    // TODO: Should be derived from image size, can't assume 16x16 tilesheet for
                    // graphical terminals
                    let font_size = image.size().as_vec2() / 16.0;
                    layout.pixels_per_tile = font_size.as_uvec2();
                    layout.tile_size = match layout.scaling {
                        TileScaling::World => {
                            let aspect = font_size.x / font_size.y;
                            Vec2::new(aspect, 1.0)
                        }
                        TileScaling::Pixels => font_size,
                    };
                    //info!("Updating layout ppt. Now {}", layout.pixels_per_tile);
                }
            }
        }
    }
}
