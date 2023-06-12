extern crate ffmpeg_next as ffmpeg;

use std::thread;
use std::time::Duration;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use image::{imageops::resize, GrayImage, ImageBuffer};
use terminal_size::{Height, Width};


fn main() -> Result<(), ffmpeg::Error> {
    let width: u16;
    let height: u16;

    if let Some((Width(w), Height(h))) = terminal_size::terminal_size() {
        width = w;
        height = h;
    } else {
        panic!("not yet implemented");
    }

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

    let mut index = 0;

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut frame = Video::empty();
                scaler.run(&decoded, &mut frame)?;

                let img: GrayImage =
                    ImageBuffer::from_raw(frame.width(), frame.height(), frame.data(0).to_vec())
                        .unwrap();

                let small_img = resize(
                    &img,
                    width.clone().into(),
                    height.try_into().unwrap(),
                    image::imageops::FilterType::Nearest,
                );

                small_img.save(format!("dump{}.png", index).to_string());
                index += 1;

                for r in 0..small_img.height() {
                    for c in 0..small_img.width() {
                        print!(
                            "{}",
                            map_gray_level_to_ascii(small_img.get_pixel(c, r).0[0])
                        );
                    }
                    println!();
                }
                thread::sleep(Duration::from_millis(16));
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

    Ok(())
}

fn map_gray_level_to_ascii(gray_level: u8) -> char {
    let ascii_scale = " .:-=+*#%@";
    let gray_scale = gray_level as f32 / 255.0;
    let index = (gray_scale * (ascii_scale.len() - 1) as f32).round() as usize;
    ascii_scale.chars().nth(index).unwrap()
}

fn draw_image() {}
