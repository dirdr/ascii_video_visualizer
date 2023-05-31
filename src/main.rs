extern crate ffmpeg_next as ffmpeg;

use std::time::Duration;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;

fn main() -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();

    // open the file
    let mut ictx = input(&"./resources/drift.mp4")?;

    // find the best video flux
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let video_stream_index = input.index();

    // find the decoder
    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::GRAY8,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let mut images: Vec<Vec<u8>> = Vec::new();

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut gray_frame = Video::empty();
                scaler.run(&decoded, &mut gray_frame)?;
                let image: Vec<u8> = gray_frame.data(0).into();
                images.push(image);
            }
            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            receive_and_process_decoded_frames(&mut decoder)?;
        }
    }
    decoder.send_eof()?;
    receive_and_process_decoded_frames(&mut decoder)?;

    let ascii_frames: Vec<Vec<char>> = images
        .iter()
        .map(|image| convert_images_to_ascii(image.clone()))
        .collect();

    print_buffer(decoder.width(), decoder.height(), ascii_frames);

    Ok(())
}

fn map_gray_level_to_ascii(gray_level: u8) -> char {
    let ascii_scale = " .:-=+*#%@";
    let gray_scale = gray_level as f32 / 255.0;
    let index = (gray_scale * (ascii_scale.len() - 1) as f32).round() as usize;
    ascii_scale.chars().nth(index).unwrap()
}

fn convert_images_to_ascii(gray_value_image: Vec<u8>) -> Vec<char> {
    gray_value_image
        .iter()
        .cloned()
        .map(|e| map_gray_level_to_ascii(e))
        .collect::<Vec<char>>()
}

fn print_buffer(width: u32, height: u32, buffer: Vec<Vec<char>>) {
    let fps = 60;
    let sleep_time: u64 = (1 / fps) * 1000;

    for image in buffer {
        println!("test");
        std::thread::sleep(Duration::from_millis(sleep_time));
    }
}
