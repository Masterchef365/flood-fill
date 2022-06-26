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
    let mut labels_copy = labels.clone();

    let clear = 0xff;

    let start = Instant::now();
    optimize_image(&mut labels, clear);
    let opt_elapsed = start.elapsed();

    write_netpbm(
        "out.pgm",
        &labels.data,
        width as usize,
        ImageChannels::Grayscale,
    )?;

    //let rect = floodfill::fill(873, 377, 0, &mut labels);
    //let rect = floodfill::fill(0, 0, 255, &mut labels);

    let start = Instant::now();
    let bboxes = floodfill::bboxes(clear, &mut labels);
    let bbox_elapsed = start.elapsed();

    for bbox in &bboxes {
        println!("{:?}", bbox);
    }

    eprintln!("Opt time: {}ms", opt_elapsed.as_secs_f32() * 1000.);
    eprintln!("Bbox time: {}ms", bbox_elapsed.as_secs_f32() * 1000.);
    eprintln!(
        "Total time: {}ms",
        (bbox_elapsed + opt_elapsed).as_secs_f32() * 1000.
    );

    for (bbox, _) in &bboxes {
        draw_bbox(&mut rgb, *bbox);
    }

    write_netpbm("out.ppm", &rgb.data, width as usize, ImageChannels::Rgb)?;

    println!("Checking against original...");

    let start = Instant::now();
    let no_opt_bboxes = floodfill::bboxes(clear, &mut labels_copy);
    let no_opt_bbox_elapsed = start.elapsed();
    assert_eq!(no_opt_bboxes, bboxes);
    eprintln!("No opt time: {}ms", no_opt_bbox_elapsed.as_secs_f32() * 1000.);

    Ok(())
}

// /// Fill contiguous areas with clear color if and only if they are surrounded on all sides
// /// (including corners) with the same color. This saves time in the clearing code, since there is
// /// not as much unexplored, non-contiguous geometry.

/// What the fuck
fn optimize_image(img: &mut MinimalImage, clear: u8) {
    let tile_w = 1;
    let tile_h = 1;

    let mut fill_coords = vec![];

    // Discover tiles
    for tile_row in 1..(img.height() / tile_h) - 1 {
        let row = tile_row * tile_h;

        if tile_row & 1 == 0 { continue }

        for tile_col in 1..(img.width() / tile_w) - 1 {
            let col = tile_col * tile_w;

            //if tile_col & 1 == 0 { continue }

            let mut all_match = true;
            let px = img.row(row)[col];

            for y in (row - 1..).take(tile_h + 2) {
                let r = &img.row(y)[col - 1..][..tile_w + 2];
                all_match &= r.iter().all(|&mem| px == mem);
                if !all_match {
                    break;
                };
            }

            if all_match {
                fill_coords.push((col, row));
            }
        }
    }

    // Draw tiles
    for (col, row) in fill_coords {
        for y in (row..).take(tile_h) {
            img.row_mut(y)[col..][..tile_w].fill(clear);
        }
    }
}

const LABEL_CHANNEL_IDX: usize = 0;

#[derive(Clone)]
pub struct MinimalImage {
    pub n_channels: u32,
    pub width: u32,
    pub data: Vec<u8>,
}

impl MinimalImage {
    pub fn row(&self, row: usize) -> &[u8] {
        &self.data[row * self.row_size()..(row + 1) * self.row_size()]
    }

    pub fn row_mut(&mut self, row: usize) -> &mut [u8] {
        let range = row * self.row_size()..(row + 1) * self.row_size();
        &mut self.data[range]
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
        tiff::ColorType::RGB(_) => 3,
        //tiff::ColorType::Other(depths) => depths.len() as u32,
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
