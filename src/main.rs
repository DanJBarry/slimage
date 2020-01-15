#![allow(clippy::all)]
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate image;

mod decoder;
mod graphics;

use std::{sync::mpsc::channel, thread};

use decoder::DecoderMessage;
use image::RgbaImage;

pub fn main() {
    let (decoder_send, decoder_recv) = channel::<DecoderMessage>();
    let (graphics_send, graphics_recv) = channel::<RgbaImage>();

    let dthread = thread::spawn(move || {
        decoder::Decoder::init(decoder_recv, graphics_send);
    });
    graphics::init(graphics_recv, decoder_send);
    dthread.join().expect("Decoder thread failed");
}
