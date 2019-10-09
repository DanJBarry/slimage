#![allow(clippy::all)]
#[macro_use] extern crate gfx;
extern crate glutin;
extern crate gfx_device_gl;
extern crate image;

mod decoder;
mod graphics;

use std::{thread, sync::mpsc::channel};

use image::RgbaImage;
use decoder::DecoderMessage;

pub fn main() {
    let (decoder_send, decoder_recv) = channel::<DecoderMessage>();
    let (graphics_send, graphics_recv) = channel::<RgbaImage>();

    let gthread = thread::spawn(move || {
        graphics::init(graphics_recv, decoder_send);
    });
    decoder::Decoder::init(decoder_recv, graphics_send);
    gthread.join().expect("Window thread failed");
}
