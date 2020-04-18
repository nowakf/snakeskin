use super::{Rect, Stack};
use std::cmp::min;

pub fn sample_random(stack: &Stack, bounds: Rect) -> Vec<u32> {
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

pub fn sample_all(stack: &Stack, bounds: Rect) -> Vec<u32> {
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
pub fn bounded_block(stack: &Stack, index: u32, block_width: u32) -> Rect {
    let x = index % stack.w;
    let y = index / stack.w;
    let h_width = block_width / 2;

    let x = x.checked_sub(h_width).unwrap_or(0);
    let right = min(x + h_width, (stack.buffer.len() - 1) as u32);
    let y = y.checked_sub(h_width).unwrap_or(0);
    let bottom = min(y + h_width, stack.buffer.len() as u32 / stack.w - 1);
    let w = right - x;
    let h = bottom - y;
    Rect{x, y, w, h}
}
