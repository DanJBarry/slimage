#[macro_use] extern crate gfx;

extern crate gfx_core;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use gfx::{
    Encoder,
    format::{
        Rgba8,
        DepthStencil,
    },
};
use gfx_core::{
    Device,
};
use gfx_device_gl::{
    CommandBuffer,
    Resources,
};
use glutin::{
    ContextBuilder,
    dpi::{
        PhysicalSize,
        LogicalSize,
    },
    EventsLoop,
    WindowBuilder,
};
use image::{
    DynamicImage,
    GenericImageView,
};

pub type ColorFormat = Rgba8;
pub type DepthFormat = DepthStencil;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn open_image_from_args() -> DynamicImage {
    use std::{
        env::args_os,
        path::Path,
    };
    let path_str = match args_os().nth(1) {
        None => panic!(),
        Some(i) => i,
    };
    image::open(Path::new(&path_str)).unwrap()
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let img = open_image_from_args();
    let builder = WindowBuilder::new()
        .with_dimensions(LogicalSize::from_physical(PhysicalSize::from(img.dimensions()), 1.0));
    let context = ContextBuilder::new();
    let (window, mut device, mut factory, mut target, mut depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop)
        .unwrap();
    let mut encoder: Encoder<Resources, CommandBuffer> = factory.create_command_buffer().into();

    let mut running = true;
    while running {
        events_loop.poll_events(|event: glutin::Event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                use glutin::WindowEvent::*;
                match event {
                    KeyboardInput { .. } | CloseRequested => running = false,
                    Resized(_) => {
                        gfx_window_glutin::update_views(&window, &mut target, &mut depth);
                    },
                    _ => (),
                }
            }
        });

        encoder.clear(&target, BLACK);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
