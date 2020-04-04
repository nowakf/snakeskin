use image::{GenericImage, ImageBuffer, DynamicImage, GrayImage, SubImage};
use anyhow::Result;

use std::fs;
use std::path::Path;
use std::cmp::{min, max};

struct Stack {
    w: u32,
    h: u32,
    buffer: Vec<u8>,
}

struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

fn load_images<P: AsRef<Path>>(path: P) -> Result<Vec<GrayImage>> {
    let mut images = Vec::new();

    for entry in fs::read_dir(path)? {
        let e = entry?;
        let name = e.file_name();
        if name.to_str().unwrap().ends_with("jpg") {
            let img = image::open(e.path())?;
            images.push(img.to_luma());
        }
    }

    Ok(images)
}
fn merge(stack: Vec<GrayImage>) -> Stack {
    let mut w = std::u32::MAX;
    let mut h = std::u32::MAX;
    for im in stack.iter() {
        w = min(im.width(), w);
        h = min(im.height(), h);
    }
    let mut buffer = Vec::new();
    for mut im in stack {
        //crop then add
        let extrax = im.width() - w;
        let extray = im.height() - h;
        buffer.append(&mut im.sub_image(extrax / 2, extray /2, w, h).to_image().into_raw());
    }
    Stack{w, h, buffer}
}
fn sample_random(stack: &Stack, bounds: Rect) -> Vec<u32> {
    use rand::RngCore;
    let mut bins = vec![0u32; 256];
    let mut rng = rand::thread_rng();
    let mut buf = vec![0u8; (bounds.w * bounds.h / 2) as usize];
    rng.fill_bytes(&mut *buf);
    while let (Some(x), Some(y)) = (buf.pop(), buf.pop()) {
        let x = bounds.x + (x as u32 % bounds.w);
        let y = bounds.y + (y as u32 % bounds.h);

        bins[stack.buffer[(y*stack.w+x) as usize] as usize] += 1;
    }
    bins
}

fn sample_all(stack: &Stack, bounds: Rect) -> Vec<u32> {
    let mut bins = vec![0u32; 256];
    let mut slices = Vec::new();

    for i in bounds.y..bounds.y + bounds.h {
        let offset = i * stack.w;
        let l = (bounds.x + offset) as usize;
        let r = (bounds.x + bounds.w + offset) as usize;
        slices.push(&stack.buffer[l..r]);
    }
    for &i in slices.iter().cloned().flatten() {
        bins[i as usize] += 1;
    }

    bins
}
fn bounded_block(stack: &Stack, index: u32, block_width: u32) -> Rect {
    let x = index % stack.w;
    let y = index / stack.w;
    let h_width = block_width / 2;

    let x = x.checked_sub(h_width).unwrap_or(0);
    let right = min(x + h_width, (stack.buffer.len() - 1) as u32);
    let y = y.checked_sub(h_width).unwrap_or(0);
    let bottom = min(y + h_width, (stack.buffer.len() as u32 / stack.w - 1));
    let w = right - x;
    let h = bottom - y;
    Rect{x, y, w, h}
}


fn entropy(stack: &Stack, sample: Box<dyn Fn(&Stack, Rect) -> Vec<u32>>, index: u32, quality: u32) -> f32
{
    let bounds = bounded_block(stack, index, quality);
    let bins = sample(stack, bounds);
    bins.iter().fold(0f32, |sum, &val| {
        let Px = (val as f32) / (stack.w * stack.h) as f32;
        if Px > 0.0 {
            sum - Px * Px.ln()
        } else {
            sum
        }
    })
}




fn filter_stack(stack: &Stack, quality: u32) -> Vec<u8> {
    let mut luma = vec![(0f32, 0f32); (stack.w * stack.h) as usize];
    for (i, &pixel) in stack.buffer.iter().enumerate() {
        let index = i as u32 % (stack.w * stack.h);
        let e = entropy(stack, Box::new(sample_random), index, quality);
        luma[index as usize].0 += (pixel as f32 / 255.0) * 1.0/e;
        luma[index as usize].1 += 1.0/e;
    }
    luma.iter().map(|(pixel, total_entropy)| (pixel / total_entropy * 255.0) as u8).collect()
}

fn main() -> Result<()> {
    let args : Vec<String> = std::env::args().collect();
    let quality : u32 = args.get(1).unwrap_or(&"3".to_string()).parse()?;
    let images = load_images("./images")?;
    let stack = merge(images);
    let out = filter_stack(&stack, quality);
    let date = chrono::Utc::now().format("%I-%M-%S-%d-%b-%Y");
    image::save_buffer(format!("./out/out_{}.png", date), &out, stack.w, stack.h, image::ColorType::L8).unwrap();
    Ok(())
}
