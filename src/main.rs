#![allow(clippy::all)]
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate image;

mod decoder;
mod graphics;

use std::{env::args_os, sync::mpsc::channel, thread};

use decoder::DecoderMessage;
use image::RgbaImage;

pub fn main() {
    let (decoder_send, decoder_recv) = channel::<DecoderMessage>();
    let (graphics_send, graphics_recv) = channel::<RgbaImage>();

    let image_path = args_os().nth(1).expect("Failed to parse arguments");
    let dimensions = image::image_dimensions(&image_path).expect("Failed to get image dimensions");
    let dthread = thread::spawn(move || {
        decoder::Decoder::init(decoder_recv, graphics_send);
    });
    decoder_send
        .send(DecoderMessage::Open(image_path))
        .expect("Failed to send message to decoder");
    graphics::init(dimensions, graphics_recv, decoder_send);
    dthread.join().expect("Decoder thread failed");
}
