use image::{GenericImage,GrayImage};
use anyhow::Result;

use std::fs;
use std::path::Path;
use std::cmp::min;

mod samplers;


//Stack is just a stack of images
//concatenated into a single array
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

//this returns an entropy weighted average for a given pixel, using the 'sample' function
//to compute the 'neighborhood'
fn entropy(stack: &Stack, sample: Box<dyn Fn(&Stack, Rect) -> Vec<u32>>, index: u32, quality: u32) -> f32
{
    let bounds = samplers::bounded_block(stack, index, quality);
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

//this applies a function to the entire stack
fn filter_stack(stack: &Stack, quality: u32) -> Vec<u8> {
    let mut luma = vec![(0f32, 0f32); (stack.w * stack.h) as usize];
    for (i, &pixel) in stack.buffer.iter().enumerate() {
        let index = i as u32 % (stack.w * stack.h);
        let e = entropy(stack, Box::new(samplers::sample_random), index, quality);
        luma[index as usize].0 += (pixel as f32 / 255.0) * 1.0/e;
        luma[index as usize].1 += 1.0/e;
    }
    luma.iter().map(|(pixel, total_entropy)| (pixel / total_entropy * 255.0) as u8).collect()
}

fn main() -> Result<()> {Q
    //parse command-line arguments
    let args : Vec<String> = std::env::args().collect();
    let quality : u32 = args.get(1).unwrap_or(&"3".to_string()).parse()?;
    //load images from folder
    let images = load_images("./images")?;
    //concatenate
    let stack = merge(images);
    //apply the transformation
    let out = filter_stack(&stack, quality);
    //output with the date in the name - so it doesn't overwrite previous files
    let date = chrono::Utc::now().format("%I-%M-%S-%d-%b-%Y");
    image::save_buffer(format!("./out/out_{}.png", date), &out, stack.w, stack.h, image::ColorType::L8).unwrap();
    Ok(())
}
