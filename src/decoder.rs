#![allow(clippy::all)]

extern crate image;

use std::{
    ffi::OsString,
    sync::mpsc::{Receiver, Sender},
};

use image::{DynamicImage, RgbaImage};

pub enum DecoderMessage {
    Open(OsString),
    RotateClockwise,
    RotateCounterclockwise,
    CloseRequested,
}

pub struct Decoder {
    image: Option<DynamicImage>,
    sender: Sender<RgbaImage>,
}

impl Decoder {
    pub fn init(receiver: Receiver<DecoderMessage>, sender: Sender<RgbaImage>) {
        let mut decoder = Decoder {
            image: None,
            sender,
        };
        loop {
            match receiver
                .recv()
                .expect("Decoder failed to receive a message")
            {
                DecoderMessage::Open(path) => {
                    decoder.open(path);
                }
                DecoderMessage::RotateClockwise => decoder.rotate_clockwise(),
                DecoderMessage::RotateCounterclockwise => decoder.rotate_counterclockwise(),
                DecoderMessage::CloseRequested => {
                    break;
                }
            }
        }
    }

    fn open(&mut self, path: OsString) {
        let result = image::open(path).expect("Failed to open image");
        self.sender
            .send(result.to_rgba())
            .expect("Failed to send data to renderer");
        self.image = Some(result);
    }

    fn rotate_clockwise(&mut self) {
        if let Some(img) = &self.image {
            let rotated = img.rotate90();
            self.sender
                .send(rotated.to_rgba())
                .expect("Failed to send data to renderer");
            self.image = Some(rotated);
        }
    }

    fn rotate_counterclockwise(&mut self) {
        if let Some(img) = &self.image {
            let rotated = img.rotate270();
            self.sender
                .send(rotated.to_rgba())
                .expect("Failed to send data to renderer");
            self.image = Some(rotated);
        }
    }
}
