use anyhow::{bail, Context, Result};
use std::fs::File;
use std::path::Path;

fn main() -> Result<()> {
    let path = std::env::args().skip(1).next().context("Requires path")?;
    let img = read_image(path)?;

    let width = img.width;

    let rgb = sample_img_channels(&img, &[0, 1, 2]);
    let labels = sample_img_channels(&img, &[LABEL_CHANNEL_IDX]);

    write_netpbm("out.pgm", &labels, width as usize, ImageChannels::Grayscale)?;
    write_netpbm("out.ppm", &rgb, width as usize, ImageChannels::Rgb)?;

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

pub fn write_netpbm(path: &str, image: &[u8], width: usize, channels: ImageChannels) -> std::io::Result<()> {
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

pub fn sample_img_channels(
    image: &MinimalImage,
    channels: &[usize],
) -> Vec<u8> {
    let mut data = vec![];
    for row in 0..image.height() {
        let row = image.row(row);
        for pixel in row.chunks_exact(image.n_channels as _) {
            for &channel in channels {
                data.push(pixel[channel]);
            }
        }
    }
    data
}

