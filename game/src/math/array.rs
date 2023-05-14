use crate::math::{Rect, Tile};
use std::ops::Add;

pub trait Array<T> {
    fn copy(&self, width: usize, rect: Rect) -> Vec<T>;

    fn paste(&mut self, width: usize, rect: Rect, data: &[T]);

    fn add(&mut self, width: usize, rect: Rect, data: &[T]);
}

impl<T> Array<T> for Vec<T>
where
    T: Copy + Default + Add<Output = T>,
{
    fn copy(&self, width: usize, rect: Rect) -> Vec<T> {
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

    fn paste(&mut self, width: usize, rect: Rect, src: &[T]) {
        let [x, y, w, h] = rect;
        for i in 0..h {
            let src_offset = i * w;
            let dst_offset = x + (y + i) * width;
            let src_range = src_offset..(src_offset + w);
            let dst_range = dst_offset..(dst_offset + w);
            self[dst_range].copy_from_slice(&src[src_range])
        }
    }

    fn add(&mut self, width: usize, rect: Rect, src: &[T]) {
        let [x, y, w, h] = rect;
        for i in 0..h {
            let src_offset = i * w;
            let dst_offset = x + (y + i) * width;
            let mut src_range = src_offset..(src_offset + w);
            let mut dst_range = dst_offset..(dst_offset + w);
            loop {
                let s = match src_range.next() {
                    Some(index) => index,
                    None => break,
                };
                let d = dst_range.next().unwrap();
                self[d] = self[d] + src[s];
            }
        }
    }
}

pub trait ArrayIndex {
    fn fit(&self, width: usize) -> usize;
    fn rect(&self, array: [usize; 2], range: [usize; 2]) -> Rect;
}

impl ArrayIndex for Tile {
    #[inline]
    fn fit(&self, width: usize) -> usize {
        self[0] + self[1] * width
    }

    fn rect(&self, array: [usize; 2], range: [usize; 2]) -> Rect {
        let [width, height] = array;
        let [w, h] = range;
        let offset_x = w / 2;
        let offset_y = h / 2;
        let [x, y] = *self;
        let min_x = if x >= offset_x { x - offset_x } else { 0 };
        let min_y = if y >= offset_y { y - offset_y } else { 0 };
        if min_x >= width || min_y >= width {
            return [0, 0, 0, 0];
        }
        let max_x = (x + w - offset_x).min(width);
        let max_y = (y + h - offset_y).min(height);
        [min_x, min_y, max_x - min_x, max_y - min_y]
    }
}

#[cfg(test)]
mod tests {
    use crate::math::{Array, ArrayIndex};

    #[test]
    fn test_rect_full_out_of_bounds() {
        let tile = [15, 1];
        let rect = tile.rect([10, 10], [5, 3]);
        assert_eq!(rect, [0, 0, 0, 0]);
    }

    #[test]
    fn test_rect_partial_right_out_of_bounds() {
        let tile = [10, 1];
        let rect = tile.rect([10, 10], [5, 3]);
        assert_eq!(rect, [8, 0, 2, 3]);
    }

    #[test]
    fn test_rect_partial_bottom_right_of_bounds() {
        let tile = [10, 10];
        let rect = tile.rect([10, 10], [5, 3]);
        assert_eq!(rect, [8, 9, 2, 1]);
    }

    #[test]
    fn test_rect_top_left_horizontal_odd_no_out() {
        let tile = [2, 1];
        let rect = tile.rect([10, 10], [5, 3]);
        assert_eq!(rect, [0, 0, 5, 3]);
    }

    #[test]
    fn test_rect_top_left_horizontal_odd_out() {
        let tile = [1, 1];
        let rect = tile.rect([10, 10], [5, 3]);
        assert_eq!(rect, [0, 0, 4, 3]);
    }

    #[test]
    fn test_rect_right_bottom_horizontal_odd_out() {
        let tile = [9, 9];
        let rect = tile.rect([10, 10], [5, 3]);
        assert_eq!(rect, [7, 8, 3, 2]);
    }

    #[test]
    fn test_rect_top_left_square_even_no_out() {
        let tile = [2, 2];
        let rect = tile.rect([10, 10], [4, 4]);
        assert_eq!(rect, [0, 0, 4, 4]);
    }

    #[test]
    fn test_rect_top_left_square_even_out() {
        let tile = [1, 1];
        let rect = tile.rect([10, 10], [4, 4]);
        assert_eq!(rect, [0, 0, 3, 3]);
    }

    #[test]
    fn test_rect_right_bottom_square_even_out() {
        let tile = [9, 9];
        let rect = tile.rect([10, 10], [4, 4]);
        assert_eq!(rect, [7, 7, 3, 3]);
    }

    #[test]
    fn test_extract_top_left_square() {
        #[rustfmt::skip]
        let map = vec![
            1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
            2, 2, 2, 2, 2, 0, 0, 0, 0, 0,
            3, 3, 3, 3, 3, 0, 0, 0, 0, 0,
            4, 4, 4, 4, 4, 0, 0, 0, 0, 0,
            5, 5, 5, 5, 5, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let result = map.copy(10, [0, 0, 5, 5]);

        #[rustfmt::skip]
        let expected = vec![
            1, 1, 1, 1, 1,
            2, 2, 2, 2, 2,
            3, 3, 3, 3, 3,
            4, 4, 4, 4, 4,
            5, 5, 5, 5, 5,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_patch_top_left_vertical_rect() {
        let mut map = vec![0; 10 * 10];

        #[rustfmt::skip]
        let influence = vec![
            1, 1, 1, 1,
            2, 2, 2, 2,
            3, 3, 3, 3,
            4, 4, 4, 4,
            5, 5, 5, 5,
        ];
        map.paste(10, [0, 0, 4, 5], &influence);

        #[rustfmt::skip]
        let expected = vec![
            1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
            2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
            3, 3, 3, 3, 0, 0, 0, 0, 0, 0,
            4, 4, 4, 4, 0, 0, 0, 0, 0, 0,
            5, 5, 5, 5, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(map, expected);
    }

    #[test]
    fn test_append_top_left_vertical_rect() {
        let mut map = vec![0; 10 * 10];

        #[rustfmt::skip]
        let influence = vec![
            1, 1, 1,
            2, 2, 2,
            3, 3, 3,
            4, 4, 4,
        ];
        map.add(10, [0, 0, 3, 4], &influence);
        map.add(10, [0, 0, 3, 4], &influence);

        #[rustfmt::skip]
        let expected = vec![
            2, 2, 2, 0, 0, 0, 0, 0, 0, 0,
            4, 4, 4, 0, 0, 0, 0, 0, 0, 0,
            6, 6, 6, 0, 0, 0, 0, 0, 0, 0,
            8, 8, 8, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(map, expected);
    }

    #[test]
    fn test_append_bottom_right_horizontal_rect() {
        let mut map = vec![0; 10 * 10];

        #[rustfmt::skip]
        let influence = vec![
            1, 1, 1, 1,
            2, 2, 2, 2,
            3, 3, 3, 3,
        ];
        map.add(10, [6, 7, 4, 3], &influence);
        map.add(10, [6, 7, 4, 3], &influence);

        #[rustfmt::skip]
        let expected = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 2, 2, 2, 2,
            0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
            0, 0, 0, 0, 0, 0, 6, 6, 6, 6,
        ];
        assert_eq!(map, expected);
    }
}
