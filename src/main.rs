extern crate ffmpeg_next as ffmpeg;

use clap::Parser;

use crossterm::QueueableCommand;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::flag::Flags;
use ffmpeg::software::scaling::Context;
use ffmpeg::util::frame::video::Video;
use image::{imageops::resize, GrayImage, ImageBuffer};
use std::thread;
use std::time::Duration;
use terminal_size::{Height, Width};

struct TermSize {
    pub width: u32,
    pub height: u32,
}

enum Mode {
    Mean,
    Individual
}

#[derive(Parser, Debug)]
#[command(name = "ascii_video_visualizer")]
#[command(author = "Adrien P. <adrien.pelfresne@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "convert mp4 video into ascii visualisation!")]
pub struct Cli {
    #[arg(short, long, default_value = "drift.mp4")]
    pub path: String,

    pub mode: String
}

fn main() -> Result<(), ffmpeg::Error> {
    let cli = Cli::parse();
    
    let mode: Mode = match &cli.mode[..] {
        "mean" => Mode::Mean,
        "individual" => Mode::Individual,
        _ => Mode:: Individual // default value
    };

    let terminal_size = get_scaled_term_size(cli.scale).unwrap();

    let path = format!("./resources/{}", cli.path.clone());

    ffmpeg::init().unwrap();


    let mut stdout = std::io::stdout();

    stdout.queue(crossterm::cursor::Hide).ok();

    // open the file
    let mut ictx = input(&path)?;

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

    let mut frame: Vec<Vec<char>> =
        vec![vec![' '; terminal_size.width as usize]; terminal_size.height as usize];

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            receive_and_process_decoded_frames(
                &mut decoder,
                &mut scaler,
                &terminal_size,
                &mut frame,
            )?;
        }
    }

    decoder.send_eof()?;
    receive_and_process_decoded_frames(&mut decoder, &mut scaler, &terminal_size, &mut frame)?;


    stdout.queue(crossterm::cursor::Hide).ok();
    Ok(())
}

fn receive_and_process_decoded_frames(
    decoder: &mut ffmpeg::decoder::Video,
    scaler: &mut Context,
    terminal_size: &TermSize,
    frame_vec: &mut Vec<Vec<char>>,
) -> Result<(), ffmpeg::Error> {
    let mut decoded = Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut frame = Video::empty();
        let mut new_frame_vec: Vec<Vec<char>> =
            vec![vec![' '; terminal_size.width as usize]; terminal_size.height as usize];

        scaler.run(&decoded, &mut frame)?;

        let img: GrayImage =
            ImageBuffer::from_raw(frame.width(), frame.height(), frame.data(0).to_vec()).unwrap();

        let small_img = resize(
            &img,
            terminal_size.width.clone(),
            terminal_size.height.clone(),
            image::imageops::FilterType::Nearest,
        );

        for r in 0..terminal_size.height as u32 {
            for c in 0..terminal_size.width as u32 {
                let char = map_gray_level_to_ascii(small_img.get_pixel(c, r).0[0]);
                new_frame_vec[r as usize][c as usize] = char;
                let old_char = frame_vec[r as usize][c as usize];
                if char != old_char {
                    print!(
                        "{}{}",
                        termion::cursor::Goto((c + 1) as u16, (r + 1) as u16),
                        char
                    );
                }
            }
        }

        std::mem::swap(frame_vec, &mut new_frame_vec);
        thread::sleep(Duration::from_millis(16));
    }
    Ok(())
}

fn get_scaled_term_size(scale: f64) -> Option<TermSize> {
    if let Some((Width(w), Height(h))) = terminal_size::terminal_size() {
        return Some(TermSize {
            width: w as u32,
            height: h as u32,
        });
    }
    None
}

fn map_gray_level_to_ascii(gray_level: u8) -> char {
    let ascii_scale = " .:-=+*#%@";
    let gray_scale = gray_level as f32 / 255.0;
    let index = (gray_scale * (ascii_scale.len() - 1) as f32).round() as usize;
    ascii_scale.chars().nth(index).unwrap()
}
