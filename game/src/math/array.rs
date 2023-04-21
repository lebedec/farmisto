use crate::math::{Rect, Tile};
use log::info;

pub trait Array2D<T> {
    fn extract_rect(&self, width: usize, rect: Rect) -> Vec<T>;
    fn patch_rect(&mut self, width: usize, rect: Rect, data: Vec<T>);
}

impl<T> Array2D<T> for Vec<T>
where
    T: Copy + Default,
{
    fn extract_rect(&self, width: usize, rect: Rect) -> Vec<T> {
        let [x, y, w, h] = rect;
        let mut dst = vec![T::default(); w * h];
        for i in 0..h {
            let src_offset = x + (y + i) * width;
            let dst_offset = i * w;
            let src_range = src_offset..(src_offset + w);
            let dst_range = dst_offset..(dst_offset + w);
            dst[dst_range].copy_from_slice(&self[src_range]);
        }
        dst
    }

    fn patch_rect(&mut self, width: usize, rect: Rect, src: Vec<T>) {
        let [x, y, w, h] = rect;
        for i in 0..h {
            let src_offset = i * w;
            let dst_offset = x + (y + i) * width;
            let src_range = src_offset..(src_offset + w);
            let dst_range = dst_offset..(dst_offset + w);
            self[dst_range].copy_from_slice(&src[src_range])
        }
    }
}
