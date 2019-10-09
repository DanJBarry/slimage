#![allow(clippy::all)]

extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use std::{convert::TryInto, env::args_os, sync::mpsc::{Receiver, Sender}};

use crate::decoder::DecoderMessage;

use gfx::{Device, Factory, traits::FactoryExt};
use glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent, ElementState};
use image::RgbaImage;

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

pub fn init(
    receiver: Receiver<RgbaImage>,
    sender: Sender<DecoderMessage>,
) {
    sender.send(DecoderMessage::Open(args_os().nth(1).expect("Failed to parse arguments")))
        .expect("Failed to send message to decoder");

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("slimage".to_string())
        .with_dimensions((500, 500).into());
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
    let pso = factory.create_pipeline_simple(&vs_code, &fs_code, pipe::new())
        .unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let transform_buffer = factory.create_constant_buffer(1);

    let image = receiver.recv().expect("Failed to receive data from decoder");
    let mut dimensions = image.dimensions();
    let mut kind = gfx::texture::Kind::D2(dimensions.0.try_into().unwrap(), dimensions.1.try_into().unwrap(), gfx::texture::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<ColorFormat>(
        kind,
        gfx::texture::Mipmap::Provided,
        &[&image],
    ).expect("Failed to create image texture");
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
                    } => {
                        sender.send(DecoderMessage::CloseRequested)
                            .expect("Failed to send close message to decoder");
                        running = false
                    },
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            ..
                        },
                        ..
                    } => {
                        sender.send(DecoderMessage::RotateCounterclockwise)
                            .expect("Failed to send close message to decoder");
                    },
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::E),
                            ..
                        },
                        ..
                    } => {
                        sender.send(DecoderMessage::RotateClockwise)
                            .expect("Failed to send close message to decoder");
                    },
                    WindowEvent::Resized(size) => {
                        window_ctx.resize(size.to_physical(window_ctx.window().get_hidpi_factor()));
                        gfx_window_glutin::update_views(&window_ctx, &mut data.out, &mut main_depth);
                    },
                    _ => (),
                }
            }
        });

        if let Ok(image) = receiver.try_recv() {
            dimensions = image.dimensions();
            kind = gfx::texture::Kind::D2(dimensions.0.try_into().unwrap(), dimensions.1.try_into().unwrap(), gfx::texture::AaMode::Single);
            let (_, view) = factory.create_texture_immutable_u8::<ColorFormat>(
                kind,
                gfx::texture::Mipmap::Provided,
                &[&image],
            ).expect("Failed to create image texture");
            data.tex.0 = view;
        }

        // draw a frame
        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.update_buffer(&data.transform, &[TRANSFORM], 0).expect("Failed to update buffer");
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window_ctx.swap_buffers().unwrap();
        device.cleanup();
    }
}
