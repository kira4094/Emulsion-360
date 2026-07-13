use std::io::Write;
use std::path::PathBuf;
use gelatin::cgmath::{Matrix4, Deg, perspective};
use gelatin::misc::LogicalRect;
use gelatin::glium::texture::RawImage2d;
use gelatin::glium::{
    self, uniform,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction},
    texture::SrgbTexture2d,
    Frame, Program, Surface, VertexBuffer, IndexBuffer,
};
use gelatin::DrawContext;
use crate::image_cache::AnimationFrameTexture;
use crate::shaders;

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
    program: Program,
    full_texture: Option<SrgbTexture2d>,
    /// File path of the currently loaded 360 image
    current_path: Option<PathBuf>,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub is_active: bool,
}

impl SphereViewer {
    pub fn new(display: &gelatin::Display) -> Self {
        let program = gelatin::shaders::shader_from_source(
            display, gelatin::shaders::ShaderDescriptor {
                vertex_shader: shaders::SPHERE_VERTEX_140,
                fragment_shader: shaders::SPHERE_FRAGMENT_140,
                outputs_srgb: false, ..Default::default()
            },
        ).unwrap();
        let (verts, idx) = build_sphere_mesh(SPHERE_COLS, SPHERE_ROWS);
        let vb = VertexBuffer::new(display, &verts).unwrap();
        let ib = IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &idx).unwrap();
        SphereViewer {
            vertex_buffer: vb, index_buffer: ib, program,
            full_texture: None, current_path: None,
            yaw: 0.0, pitch: 0.0, fov: 90.0, is_active: false,
        }
    }

    pub fn reset_view(&mut self) {
        self.yaw = 0.0; self.pitch = 0.0; self.fov = 90.0;
    }

    /// Load a panorama directly from file as a single texture.
    pub fn load_panorama(&mut self, display: &gelatin::Display, path: &PathBuf) {
        if self.current_path.as_ref() == Some(path) && self.full_texture.is_some() {
            return; // Already loaded
        }
        match gelatin::image::open(path) {
            Ok(img) => {
                let rgba = img.into_rgba8();
                let dims = rgba.dimensions();
                let raw = RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dims);
                let tex = SrgbTexture2d::new(display, raw).unwrap();
                self.full_texture = Some(tex);
                self.current_path = Some(path.clone());
                eprintln!("[360] Loaded panorama from file: {}x{}", dims.0, dims.1);
                std::io::stderr().flush().unwrap();
            }
            Err(e) => {
                eprintln!("[360] Failed to load panorama: {}", e);
                std::io::stderr().flush().unwrap();
            }
        }
    }

    pub fn draw(
        &self, target: &mut Frame, context: &DrawContext,
        _texture: &AnimationFrameTexture, _bright_shade: f32, logical_bounds: &LogicalRect,
    ) {
        let vp = context.logical_rect_to_viewport(logical_bounds);
        if vp.width == 0 || vp.height == 0 { return; }
        let tex = match self.full_texture {
            Some(ref t) => t,
            None => return,
        };

        let draw_params = glium::DrawParameters {
            viewport: Some(vp),
            backface_culling: glium::BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };
        let aspect = vp.width as f32 / vp.height as f32;
        let proj = perspective(Deg(self.fov), aspect, 0.1_f32, 100.0_f32);
        let view = Matrix4::from_angle_x(Deg(self.pitch))
            * Matrix4::from_angle_y(Deg(self.yaw));
        let mvp = proj * view;

        let sampler = tex.sampled()
            .magnify_filter(MagnifySamplerFilter::Linear)
            .minify_filter(MinifySamplerFilter::Linear)
            .wrap_function(SamplerWrapFunction::Clamp);

        let uniforms = uniform! {
            matrix: Into::<[[f32; 4]; 4]>::into(mvp),
            tex: sampler,
            u_uv_offset: (0.0_f32, 0.0_f32),
            u_uv_scale: (1.0_f32, 1.0_f32),
        };
        target.draw(&self.vertex_buffer, &self.index_buffer,
            &self.program, &uniforms, &draw_params).unwrap();
    }
}

fn build_sphere_mesh(cols: u32, rows: u32) -> (Vec<SphereVertex>, Vec<u32>) {
    let mut verts = Vec::new();
    let mut idx = Vec::new();
    for row in 0..=rows {
        let fv = row as f32 / rows as f32;
        let lat = (0.5 - fv) * std::f32::consts::PI;
        let y = lat.sin();
        let r = lat.cos();
        for col in 0..=cols {
            let fu = col as f32 / cols as f32;
            let lon = fu * 2.0 * std::f32::consts::PI;
            verts.push(SphereVertex {
                position: [r * lon.sin(), y, r * lon.cos()],
                tex_coords: [1.0 - fu, 1.0 - fv],
            });
        }
    }
    for row in 0..rows {
        for col in 0..cols {
            let p0 = row * (cols + 1) + col;
            let p1 = p0 + cols + 1;
            idx.extend_from_slice(&[p0, p1, p0 + 1, p1, p1 + 1, p0 + 1]);
        }
    }
    (verts, idx)
}

pub fn is_panorama(texture: &AnimationFrameTexture) -> bool {
    let (w, h) = texture.oriented_dimensions();
    if w == 0 || h == 0 { return false; }
    let ratio = w as f32 / h as f32;
    (ratio - 2.0).abs() < 0.05 && w >= 2048
}