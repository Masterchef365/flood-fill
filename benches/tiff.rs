use anyhow::{bail, Result};
use floodfill::Image;
use std::fs::File;
use std::path::Path;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let images = [
        "images/10363723.TOP.DeepTraining.MoreGeo.HHR.tif",
        "images/10363824.TOP.DeepTraining.MoreGeo.HHR.tif",
    ];

    for path in images {
        let img = read_image(path).unwrap();
        let labels = sample_img_channels(&img, &[LABEL_CHANNEL_IDX]);

        c.bench_with_input(BenchmarkId::new("input_example", path), &labels, |b, s| {
            b.iter(|| floodfill::bboxes(0xff, &mut s.clone()));
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

const LABEL_CHANNEL_IDX: usize = 19;

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
