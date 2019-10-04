#![allow(clippy::all)]
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use gfx::{Device, Factory, traits::FactoryExt};
use glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use image::{DynamicImage, GenericImageView};

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 3] = "a_Color",
    }

    constant Transform {
        transform: [[f32; 4];4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

const SQUARE: &[Vertex] = &[
    Vertex { pos: [1.0, -1.0], uv: [1.0, 1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0, -1.0], uv: [0.0, 1.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [-1.0, 1.0], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { pos: [1.0, 1.0], uv: [1.0, 0.0], color: [1.0, 1.0, 1.0] },
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
const TRANSFORM: Transform = Transform {
    transform: [[1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]]
};

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

fn open_image_from_args() -> DynamicImage {
    use std::{
        env::args_os,
        path::Path,
    };
    let path_str = match args_os().nth(1) {
        None => panic!(),
        Some(i) => i,
    };
    image::open(Path::new(&path_str)).expect("Failed to open image")
}

pub fn main() {
    let img = open_image_from_args();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("slimage".to_string())
        .with_dimensions((width, height).into());

    let (vs_code, fs_code) =
    (
        include_bytes!("shader/150_core.glslv").to_vec(),
        include_bytes!("shader/150_core.glslf").to_vec(),
    );

    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Latest)
        .with_vsync(true);
    let (window_ctx, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop)
            .expect("Failed to create window");
    let mut encoder = gfx::Encoder::from(factory.create_command_buffer());

    let sampler = factory.create_sampler_linear();
    let (_, view) = factory.create_texture_immutable_u8::<ColorFormat>(
        kind,
        gfx::texture::Mipmap::Provided,
        &[&img.to_rgba()],
    ).expect("Failed to create image texture");
    let pso = factory.create_pipeline_simple(&vs_code, &fs_code, pipe::new())
        .unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let transform_buffer = factory.create_constant_buffer(1);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        tex: (view, sampler),
        transform: transform_buffer,
        out: main_color,
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested |
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => running = false,
                    WindowEvent::Resized(size) => {
                        window_ctx.resize(size.to_physical(window_ctx.window().get_hidpi_factor()));
                        gfx_window_glutin::update_views(&window_ctx, &mut data.out, &mut main_depth);
                    },
                    _ => (),
                }
            }
        });

        // draw a frame
        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.update_buffer(&data.transform, &[TRANSFORM], 0).expect("Failed to update buffer");
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window_ctx.swap_buffers().unwrap();
        device.cleanup();
    }
}
