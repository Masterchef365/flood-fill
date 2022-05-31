pub trait Image {
    type Pixel: Copy + PartialEq + std::fmt::Debug;

    fn get_pixel(&self, x: i32, y: i32) -> Option<Self::Pixel>;
    fn set_pixel(&mut self, x: i32, y: i32, pixel: Self::Pixel);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

impl<T: Image> Image for &mut T {
    type Pixel = T::Pixel;

    fn get_pixel(&self, x: i32, y: i32) -> Option<Self::Pixel> {
        (**self).get_pixel(x, y)
    }

    fn set_pixel(&mut self, x: i32, y: i32, pixel: Self::Pixel) {
        (**self).set_pixel(x, y, pixel)
    }

    fn width(&self) -> usize {
        (**self).width()
    }

    fn height(&self) -> usize {
        (**self).height()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rect {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

/// Fill the image with nv, and return all of the bounding boxes and their colors
pub fn bboxes<I: Image>(nv: I::Pixel, mut img: I) -> Vec<(Rect, I::Pixel)> {
    let height = img.height() as i32;
    let width = img.width() as i32;

    let mut rects = vec![];

    for y in 0..height {
        for x in 0..width {
            let color = img.get_pixel(x, y).unwrap();
            if let Some(rect) = fill(x, y, nv, &mut img) {
                rects.push((rect, color));
            }
        }
    }

    rects
}

/// https://en.wikipedia.org/wiki/Flood_fill#Span_Filling
pub fn fill<I: Image>(x: i32, y: i32, nv: I::Pixel, mut img: I) -> Option<Rect> {
    let inside = match img.get_pixel(x, y) {
        None => return None,
        Some(px) if px == nv => return None,
        Some(other) => other,
    };

    let mut rect = Rect::point(x, y);

    let mut set_pixel = |img: &mut I, x, y| {
        img.set_pixel(x, y, nv);
        rect.insert(x, y);
    };

    let mut s = Vec::new();
    s.push((x, x, y, 1)); 
    s.push((x, x, y - 1, -1));

    while let Some((mut x1, x2, y, dy)) = s.pop() {
        let mut x = x1;
        if img.get_pixel(x, y) == Some(inside) {
            while img.get_pixel(x - 1, y) == Some(inside) {
                set_pixel(&mut img, x - 1, y);
                x -= 1;
            }
        }

        if x < x1 {
            s.push((x, x1 - 1, y - dy, -dy));
        }

        while x1 <= x2 {
            while img.get_pixel(x1, y) == Some(inside) {
                set_pixel(&mut img, x1, y);
                x1 += 1;
                s.push((x, x1 - 1, y + dy, dy));
                if x1 - 1 > x2 {
                    s.push((x2 + 1, x1 - 1, y - dy, -dy));
                }
            }

            x1 += 1;

            while x1 < x2 && img.get_pixel(x1, y) != Some(inside) {
                x1 += 1
            }

            x = x1
        }
    }

    Some(rect)
}

impl Rect {
    pub fn point(x: i32, y: i32) -> Self {
        Self {
            left: x,
            right: x,
            top: y,
            bottom: y,
        }
    }

    pub fn insert(&mut self, x: i32, y: i32) {
        self.left = self.left.min(x);
        self.right = self.right.max(x);

        self.top = self.top.min(y);
        self.bottom = self.bottom.max(y);
    }
}
