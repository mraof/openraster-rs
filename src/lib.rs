use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use image::{EncodableLayout, RgbaImage};
use stack::{LayerStack, Stack};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

#[cfg(test)]
mod tests;

pub mod stack;

pub struct OpenRaster {
    pub data: HashMap<String, Vec<u8>>,
    thumbnail: Vec<u8>,
    mergedimage: Vec<u8>,
    pub stack: LayerStack,
}

impl OpenRaster {
    pub fn new(w: u32, h: u32) -> OpenRaster {
        OpenRaster {
            data: HashMap::new(),
            thumbnail: Vec::new(),
            mergedimage: Vec::new(),
            stack: LayerStack {
                version: "0.0.6".to_string(),
                w,
                h,
                xres: None,
                yres: None,
                stack: Stack::default(),
            },
        }
    }

    pub fn save(&self, f: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(f)?;
        let mut zip = ZipWriter::new(file);
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("mimetype", stored)?;
        zip.write_all(b"image/openraster")?;

        zip.start_file("stack.xml", deflate)?;
        zip.write_all(&self.stack.xml().into_bytes())?;

        zip.add_directory("Thumbnail", deflate)?;
        zip.start_file("Thumbnail/thumbnail.png", stored)?;
        zip.write_all(&self.thumbnail)?;

        zip.start_file("mergedimage.png", stored)?;
        zip.write_all(&self.mergedimage)?;

        //By convention there should only be data/ but that's only convention
        let mut created_dirs: HashSet<PathBuf> = HashSet::new();
        for (k, d) in &self.data {
            let path = PathBuf::from_str(k)?;
            println!("{path:?}");
            if let Some(dir) = path.parent()
                && !created_dirs.contains(dir) {
                    zip.add_directory_from_path(dir, deflate)?;
                    created_dirs.insert(dir.to_path_buf());
                }
            zip.start_file_from_path(path, deflate)?;
            zip.write_all(d)?;
        }
        Ok(())
    }

    pub fn render(&mut self) {
        let stack = &self.stack.stack;
        let img = self.render_stack(stack);
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, self.stack.w, self.stack.h);
            encoder.set_color(png::ColorType::Rgba);
            //mergedimage.png can also be 16 bits but I'm unclear when
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(img.as_bytes()).unwrap();
        }
        self.mergedimage = bytes;
        let thumbnail = if self.stack.w.max(self.stack.h) > 256 {
            let ratio = 256.0 / (self.stack.w.max(self.stack.h) as f64);
            let w = ((self.stack.w as f64 * ratio) as u32).min(256);
            let h = ((self.stack.h as f64 * ratio) as u32).min(256);
            image::imageops::resize(&img, w, h, image::imageops::FilterType::Nearest)
        } else {
            img
        };
        let mut thumbnail_bytes = Vec::new();
        {
            let mut encoder =
                png::Encoder::new(&mut thumbnail_bytes, thumbnail.width(), thumbnail.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(thumbnail.as_bytes()).unwrap();
        }
        self.thumbnail = thumbnail_bytes;
    }

    //Not complete or correct but not my main priority right now
    fn render_stack(&self, stack: &Stack) -> RgbaImage {
        let mut img = RgbaImage::new(self.stack.w, self.stack.h);
        for sub in stack.sub.iter().rev() {
            if sub.visible() {
                match sub {
                    stack::Sub::Layer(layer) => {
                        let bytes = self.data.get(&layer.src).unwrap();
                        let mut sub_img = if layer.src.ends_with(".svg") {
                            let width = img.width();
                            let height = img.height();
                            let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).unwrap();
                            let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default()).unwrap();
                            resvg::render(&tree, resvg::tiny_skia::Transform::identity(), &mut pixmap.as_mut());
                            RgbaImage::from_vec(width, height, Vec::from(pixmap.data())).unwrap()
                        } else {
                            image::load_from_memory(bytes).unwrap().to_rgba8()
                        };
                        if let Some(opacity) = layer.opacity {
                            for p in sub_img.pixels_mut() {
                                p[3] = (p[3] as f64 * opacity) as u8;
                            }
                        }
                        let sx = layer.x.unwrap_or(0) as i64;
                        let sy = layer.y.unwrap_or(0) as i64;
                        image::imageops::overlay(&mut img, &sub_img, sx, sy)
                    }
                    stack::Sub::Stack(sub_stack) => {
                        let sub_img = self.render_stack(sub_stack);
                        //TODO Handle isolation and composite-op properly
                        //Prior to version 0.0.6 x and y were allowed, should that be handled?
                        image::imageops::overlay(&mut img, &sub_img, 0, 0);
                    }
                }
            }
        }
        img
    }
}
