use gelatin::cgmath::{Matrix4, Deg, perspective};
use gelatin::misc::LogicalRect;
use gelatin::glium::{
	self,
	uniform,
	uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction},
	Frame, Program, Surface, VertexBuffer, IndexBuffer,
};
use std::io::Write;
use gelatin::DrawContext;

use crate::image_cache::AnimationFrameTexture;
use crate::shaders;

/// Segments of the sphere mesh (longitude × latitude).
const SPHERE_COLS: u32 = 128;
const SPHERE_ROWS: u32 = 64;

#[derive(Copy, Clone)]
struct SphereVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
glium::implement_vertex!(SphereVertex, position, tex_coords);

pub struct SphereViewer {
    vertex_buffer: VertexBuffer<SphereVertex>,
    index_buffer: IndexBuffer<u32>,
    index_count: u32,
    program: Program,
    /// Horizontal viewing angle (yaw) in degrees
    pub yaw: f32,
    /// Vertical viewing angle (pitch) in degrees
    pub pitch: f32,
    /// Horizontal field of view in degrees
    pub fov: f32,
    /// Whether the current image is a 360° panorama
    pub is_active: bool,
}

impl SphereViewer {
    pub fn new(display: &gelatin::Display) -> Self {
        let program = gelatin::shaders::shader_from_source(
            display,
            gelatin::shaders::ShaderDescriptor {
                vertex_shader: shaders::SPHERE_VERTEX_140,
                fragment_shader: shaders::SPHERE_FRAGMENT_140,
                outputs_srgb: false,
                ..Default::default()
            },
        )
        .unwrap();

        let (vertices, indices) = build_sphere_mesh(SPHERE_COLS, SPHERE_ROWS);
        let vertex_buffer = VertexBuffer::new(display, &vertices).unwrap();
        let index_buffer = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )
        .unwrap();
        let index_count = indices.len() as u32;

        SphereViewer {
            vertex_buffer,
            index_buffer,
            index_count,
            program,
            yaw: 0.0,
            pitch: 0.0,
            fov: 90.0,
            is_active: false,
        }
    }

    /// Reset camera to default orientation.
    pub fn reset_view(&mut self) {
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.fov = 90.0;
    }

    /// Draw the equirectangular panorama as a sphere with the camera inside.
    pub fn draw(
        &self,
        target: &mut Frame,
        context: &DrawContext,
        texture: &AnimationFrameTexture,
        bright_shade: f32,
        logical_bounds: &LogicalRect,
    ) {
        let vp = context.logical_rect_to_viewport(logical_bounds);
        let size_w = vp.width as f32;
        let size_h = vp.height as f32;
        if size_w <= 0.0 || size_h <= 0.0 {
            return;
        }

        eprintln!("[360] Drawing sphere: yaw={:.1} pitch={:.1} fov={:.1} size={:.0}x{:.0}",
            self.yaw, self.pitch, self.fov, size_w, size_h);
        std::io::stderr().flush().unwrap();
        let draw_params = glium::DrawParameters {
            viewport: Some(vp),
            backface_culling: glium::BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };

        // Build MVP matrix: Projection * View
        let aspect = size_w / size_h;
        let projection = perspective(Deg(self.fov), aspect, 0.1_f32, 100.0_f32);

        // Camera is at origin, looking into the sphere
        // Rotate by yaw (around Y) then pitch (around X)
        let view = Matrix4::from_angle_x(Deg(self.pitch))
            * Matrix4::from_angle_y(Deg(self.yaw));

        let mvp = projection * view;

        // Render all texture cells at their correct UV positions
        // (handles large images split into multiple GPU textures)
        let grid_cols = texture.tex_grid.iter().map(|c| c.col).max().unwrap_or(0) + 1;
        let grid_rows = texture.tex_grid.iter().map(|c| c.row).max().unwrap_or(0) + 1;

        for cell in texture.tex_grid.iter() {
            let u_off = cell.col as f32 / grid_cols as f32;
            let v_off = (grid_rows - 1 - cell.row) as f32 / grid_rows as f32;
            let u_scl = 1.0_f32 / grid_cols as f32;
            let v_scl = 1.0_f32 / grid_rows as f32;

            let sampler = cell
                .tex
                .sampled()
                .magnify_filter(MagnifySamplerFilter::Linear)
                .minify_filter(MinifySamplerFilter::Linear)
                .wrap_function(SamplerWrapFunction::Clamp);

            let uniforms = uniform! {
                matrix: Into::<[[f32; 4]; 4]>::into(mvp),
                tex: sampler,
                bright_shade: bright_shade,
                u_uv_offset: (u_off, v_off),
                u_uv_scale: (u_scl, v_scl),
            };

            target
                .draw(
                    &self.vertex_buffer,
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &draw_params,
                )
                .unwrap();
        }
    }
}

/// Generate a UV sphere with the camera on the inside.
/// Returns (vertices, indices).
///
/// The sphere is centered at origin with radius 1.
/// - longitude (u) goes from 0..1 around the sphere
/// - latitude (v) goes from 0..1 from top to bottom
/// UVs are flipped vertically so the sphere renders inside-out correctly.
fn build_sphere_mesh(cols: u32, rows: u32) -> (Vec<SphereVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for row in 0..=rows {
        let v = row as f32 / rows as f32;
        let lat = (0.5 - v) * std::f32::consts::PI; // -PI/2 to PI/2
        let y = lat.sin();
        let radius = lat.cos();

        for col in 0..=cols {
            let u = col as f32 / cols as f32;
            let lon = u * 2.0 * std::f32::consts::PI; // 0 to 2*PI
            let x = radius * lon.sin();
            let z = radius * lon.cos();

            vertices.push(SphereVertex {
                position: [x, y, z],
                tex_coords: [u, v],
            });
        }
    }

    // Triangle strip: two triangles per quad
    for row in 0..rows {
        for col in 0..cols {
            let p0 = row * (cols + 1) + col;
            let p1 = p0 + cols + 1;

            indices.push(p0);
            indices.push(p1);
            indices.push(p0 + 1);

            indices.push(p1);
            indices.push(p1 + 1);
            indices.push(p0 + 1);
        }
    }

    (vertices, indices)
}

/// Check if an image is likely a 360° equirectangular panorama.
/// Criteria: image is wider than tall (2:1 ratio) and large enough.
pub fn is_panorama(texture: &AnimationFrameTexture) -> bool {
    let (w, h) = texture.oriented_dimensions();
    if w == 0 || h == 0 {
        eprintln!("[360] Image size: {}x{} - too small", w, h);
        return false;
    }
    let ratio = w as f32 / h as f32;
    let is_pano = (ratio - 2.0).abs() < 0.05 && w >= 2048;
    eprintln!("[360] Image {}x{}, ratio={:.2}, is_panorama={}", w, h, ratio, is_pano);
    std::io::stderr().flush().unwrap();
    is_pano
}
