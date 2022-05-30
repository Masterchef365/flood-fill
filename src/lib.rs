pub trait Image {
    type Pixel: Copy + PartialEq + std::fmt::Debug;

    fn get_pixel(&self, x: i32, y: i32) -> Option<Self::Pixel>;
    fn set_pixel(&mut self, x: i32, y: i32, pixel: Self::Pixel);
}

impl<T: Image> Image for &mut T {
    type Pixel = T::Pixel;

    fn get_pixel(&self, x: i32, y: i32) -> Option<Self::Pixel> {
        (**self).get_pixel(x, y)
    }

    fn set_pixel(&mut self, x: i32, y: i32, pixel: Self::Pixel) {
        (**self).set_pixel(x, y, pixel)
    }
}

/*
/// Heckbert, Paul S (1990). "IV.10: A Seed Fill Algorithm". In Glassner, Andrew S (ed.).
/// Graphics Gems. Academic Press. pp. 275–277. ISBN 0122861663.
pub fn fill<I: Image>(
    x: i32,
    y: i32,
    nv: I::Pixel,
    mut img: I,
) {
    let ov = match img.get_pixel(x, y) {
        None => return,
        Some(ov) if ov == nv => return,
        Some(other) => other,
    };

    let mut stack = vec![];

    stack.push((y, x, x, 1));
    stack.push((y + 1, x, x, -1));

    while let Some((y, x1, x2, dy)) = stack.pop() {
        let mut x = x1;

        while img.get_pixel(x, y) == Some(ov) {
            img.set_pixel(x, y, nv);
            x -= 1;
        }

        let mut start = x + 1;

        if start < x1 {
            stack.push((y, start, x1 - 1, -dy));
        }

        x = x1 + 1;

        let no_skip = x < x1;

        while x <= x2 {

            if no_skip {
                while img.get_pixel(x, y) == Some(ov) {
                    img.set_pixel(x, y, nv);
                    x += 1;
                }

                stack.push((y, start, x-1, dy));

                if x > x2 + 1 {
                    stack.push((y, x2 + 1, x - 1, -dy));
                }
            }

            x += 1;

            while x <= x2 && img.get_pixel(x, y).unwrap() != ov {
                x += 1;
            }

            start = x;
        }
    }
}
*/

/// https://en.wikipedia.org/wiki/Flood_fill#Span_Filling
pub fn fill<I: Image>(x: i32, y: i32, nv: I::Pixel, mut img: I) {
    let inside = match img.get_pixel(x, y) {
        None => return,
        Some(px) if px == nv => return,
        Some(other) => other,
    };

    let mut s = vec![
        (x, x, y, 1),
        (x, x, y - 1, -1),
    ];

    while let Some((mut x1, x2, y, dy)) = s.pop() {
        let mut x = x1;
        if img.get_pixel(x, y) == Some(inside) {
            while img.get_pixel(x - 1, y) == Some(inside) {
                img.set_pixel(x - 1, y, nv);
                x -= 1;
            }
        }

        if x < x1 {
            s.push((x, x1 - 1, y - dy, -dy));
        }

        while x1 <= x2 {
            while img.get_pixel(x1, y) == Some(inside) {
                img.set_pixel(x1, y, nv);
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
}
