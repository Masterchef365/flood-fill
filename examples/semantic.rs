use anyhow::{bail, Context, Result};
use floodfill::{Image, Rect};
use std::fs::File;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let path = std::env::args().skip(1).next().context("Requires path")?;
    let img = read_image(path)?;

    let width = img.width;

    let mut rgb = sample_img_channels(&img, &[0, 1, 2]);
    let mut labels = sample_img_channels(&img, &[LABEL_CHANNEL_IDX]);

    write_netpbm(
        "out.pgm",
        &labels.data,
        width as usize,
        ImageChannels::Grayscale,
    )?;

    //let rect = floodfill::fill(873, 377, 0, &mut labels);
    //let rect = floodfill::fill(0, 0, 255, &mut labels);
    let start = Instant::now();
    let bboxes = floodfill::bboxes(0xff, &mut labels);
    let elapsed = start.elapsed();

    dbg!(&bboxes);
    println!("Time: {}ms", elapsed.as_secs_f32() * 1000.);

    for (bbox, _) in &bboxes {
        draw_bbox(&mut rgb, *bbox);
    }

    write_netpbm("out.ppm", &rgb.data, width as usize, ImageChannels::Rgb)?;

    Ok(())
}

const LABEL_CHANNEL_IDX: usize = 19;

pub struct MinimalImage {
    pub n_channels: u32,
    pub width: u32,
    pub data: Vec<u8>,
}

impl MinimalImage {
    pub fn row(&self, row: usize) -> &[u8] {
        &self.data[row * self.row_size()..(row + 1) * self.row_size()]
    }

    pub fn row_size(&self) -> usize {
        (self.n_channels * self.width) as usize
    }

    pub fn n_channels(&self) -> u32 {
        self.n_channels
    }

    pub fn height(&self) -> usize {
        self.data.len() / self.row_size()
    }

    pub fn calc_idx(&self, x: i32, y: i32) -> Option<usize> {
        let x_bound = x >= 0 && x < self.width as i32;
        let y_bound = y >= 0 && y < self.height() as i32;
        (x_bound && y_bound)
            .then(|| (x * self.n_channels as i32 + y * self.row_size() as i32) as usize)
    }
}

pub fn read_image(path: impl AsRef<Path>) -> Result<MinimalImage> {
    let file = std::io::BufReader::new(File::open(path)?);

    let mut limits = tiff::decoder::Limits::default();
    limits.decoding_buffer_size *= 1024;
    limits.intermediate_buffer_size *= 1024;
    limits.ifd_value_size *= 1024;
    let mut tiff = tiff::decoder::Decoder::new(file)?.with_limits(limits);

    // TODO: Check if all depths are 8
    let n_channels = match tiff.colortype()? {
        tiff::ColorType::Other(depths) => depths.len() as u32,
        _ => bail!("Only accepts u8 TIFFs"),
    };

    let (width, _height) = tiff.dimensions()?;

    let data = match tiff.read_image()? {
        tiff::decoder::DecodingResult::U8(buf) => buf,
        _ => unimplemented!(),
    };

    Ok(MinimalImage {
        n_channels,
        width,
        data,
    })
}

pub enum ImageChannels {
    Rgb,
    Grayscale,
}

pub fn write_netpbm(
    path: &str,
    image: &[u8],
    width: usize,
    channels: ImageChannels,
) -> std::io::Result<()> {
    use std::io::Write;
    let mut writer = std::fs::File::create(path)?;

    let (pixel_stride, header) = match channels {
        ImageChannels::Rgb => (3, "P6"),
        ImageChannels::Grayscale => (1, "P5"),
    };

    let height = image.len() / (width * pixel_stride);
    debug_assert_eq!(image.len() % width, 0);

    writer.write_all(format!("{}\n{} {}\n255\n", header, width, height).as_bytes())?;
    writer.write_all(image)?;
    Ok(())
}

pub fn sample_img_channels(image: &MinimalImage, channels: &[usize]) -> MinimalImage {
    let mut data = vec![];
    for row in 0..image.height() {
        let row = image.row(row);
        for pixel in row.chunks_exact(image.n_channels as _) {
            for &channel in channels {
                data.push(pixel[channel]);
            }
        }
    }

    MinimalImage {
        n_channels: channels.len() as u32,
        width: image.width,
        data,
    }
}

impl Image for MinimalImage {
    type Pixel = u8;
    fn get_pixel(&self, x: i32, y: i32) -> Option<Self::Pixel> {
        self.calc_idx(x, y).map(|idx| self.data[idx])
    }

    fn set_pixel(&mut self, x: i32, y: i32, pixel: Self::Pixel) {
        if let Some(idx) = self.calc_idx(x, y) {
            self.data[idx] = pixel;
        }
    }

    fn width(&self) -> usize {
        self.width as _
    }

    fn height(&self) -> usize {
        self.height()
    }
}

fn draw_bbox(image: &mut MinimalImage, bbox: Rect) {
    for i in bbox.top..bbox.bottom {
        image.set_pixel(bbox.left, i, 0xff);
        image.set_pixel(bbox.right, i, 0xff);
    }

    for i in bbox.left..bbox.right {
        image.set_pixel(i, bbox.top, 0xff);
        image.set_pixel(i, bbox.bottom, 0xff);
    }
}
