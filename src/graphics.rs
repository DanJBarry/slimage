#![allow(clippy::all)]

extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use std::{
    convert::TryInto,
    env::args_os,
    sync::mpsc::{Receiver, Sender},
};

use crate::decoder::DecoderMessage;

use gfx::{traits::FactoryExt, Device, Factory};
use glutin::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use image::RgbaImage;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
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
    Vertex {
        pos: [1.0, -1.0],
        uv: [1.0, 1.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, -1.0],
        uv: [0.0, 1.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0],
        uv: [0.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
const TRANSFORM: Transform = Transform {
    transform: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ],
};

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

struct Graphics {
    window_ctx: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::Window>,
    pipe_data: pipe::Data<gfx_device_gl::Resources>,
    depth: gfx::handle::DepthStencilView<
        gfx_device_gl::Resources,
        (gfx::format::D24_S8, gfx::format::Unorm),
    >,
    kind: gfx::texture::Kind,
    factory: gfx_device_gl::Factory,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    pipe_state: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    device: gfx_device_gl::Device,
}

fn setup(
    receiver: &Receiver<RgbaImage>,
    sender: &Sender<DecoderMessage>,
) -> (glutin::EventsLoop, Graphics) {
    sender
        .send(DecoderMessage::Open(
            args_os().nth(1).expect("Failed to parse arguments"),
        ))
        .expect("Failed to send message to decoder");

    let events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("slimage".to_string())
        .with_dimensions((500, 500).into());
    let (vs_code, fs_code) = (
        include_bytes!("shader/150_core.glslv").to_vec(),
        include_bytes!("shader/150_core.glslf").to_vec(),
    );
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Latest)
        .with_vsync(true);
    let (window_ctx, device, mut factory, main_color, depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop)
            .expect("Failed to create window");
    let encoder = gfx::Encoder::from(factory.create_command_buffer());
    let sampler = factory.create_sampler_linear();
    let pipe_state = factory
        .create_pipeline_simple(&vs_code, &fs_code, pipe::new())
        .unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let transform_buffer = factory.create_constant_buffer(1);

    let image = receiver
        .recv()
        .expect("Failed to receive data from decoder");
    let dimensions = image.dimensions();
    let kind = gfx::texture::Kind::D2(
        dimensions.0.try_into().unwrap(),
        dimensions.1.try_into().unwrap(),
        gfx::texture::AaMode::Single,
    );
    let (_, view) = factory
        .create_texture_immutable_u8::<ColorFormat>(kind, gfx::texture::Mipmap::Provided, &[&image])
        .expect("Failed to create image texture");
    let pipe_data = pipe::Data {
        vbuf: vertex_buffer,
        tex: (view, sampler),
        transform: transform_buffer,
        out: main_color,
    };
    (
        events_loop,
        Graphics {
            window_ctx,
            pipe_data,
            depth,
            kind,
            factory,
            encoder,
            pipe_state,
            slice,
            device,
        },
    )
}

pub fn init(receiver: Receiver<RgbaImage>, sender: Sender<DecoderMessage>) {
    let (mut events_loop, mut graphics) = setup(&receiver, &sender);
    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        sender
                            .send(DecoderMessage::CloseRequested)
                            .expect("Failed to send close message to decoder");
                        running = false
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                ..
                            },
                        ..
                    } => {
                        sender
                            .send(DecoderMessage::RotateCounterclockwise)
                            .expect("Failed to send close message to decoder");
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::E),
                                ..
                            },
                        ..
                    } => {
                        sender
                            .send(DecoderMessage::RotateClockwise)
                            .expect("Failed to send close message to decoder");
                    }
                    WindowEvent::Resized(size) => {
                        graphics.window_ctx.resize(
                            size.to_physical(graphics.window_ctx.window().get_hidpi_factor()),
                        );
                        gfx_window_glutin::update_views(
                            &graphics.window_ctx,
                            &mut graphics.pipe_data.out,
                            &mut graphics.depth,
                        );
                    }
                    _ => (),
                }
            }
        });

        if let Ok(image) = receiver.try_recv() {
            let dimensions = image.dimensions();
            graphics.kind = gfx::texture::Kind::D2(
                dimensions.0.try_into().unwrap(),
                dimensions.1.try_into().unwrap(),
                gfx::texture::AaMode::Single,
            );
            let (_, view) = graphics
                .factory
                .create_texture_immutable_u8::<ColorFormat>(
                    graphics.kind,
                    gfx::texture::Mipmap::Provided,
                    &[&image],
                )
                .expect("Failed to create image texture");
            graphics.pipe_data.tex.0 = view;
        }

        // draw a frame
        graphics.encoder.clear(&graphics.pipe_data.out, CLEAR_COLOR);
        graphics
            .encoder
            .update_buffer(&graphics.pipe_data.transform, &[TRANSFORM], 0)
            .expect("Failed to update buffer");
        graphics
            .encoder
            .draw(&graphics.slice, &graphics.pipe_state, &graphics.pipe_data);
        graphics.encoder.flush(&mut graphics.device);
        graphics.window_ctx.swap_buffers().unwrap();
        graphics.device.cleanup();
    }
}
