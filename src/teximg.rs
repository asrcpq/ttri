use image::ImageBuffer;

pub struct Teximg {
	pub color: bool,
	pub dim: [u32; 2],
	// rgba8
	pub data: Vec<u8>,
}

pub type RgbaImage = ImageBuffer<image::Rgba<u8>, Vec<u8>>;
pub type LumaImage = ImageBuffer<image::Luma<u8>, Vec<u8>>;

impl Teximg {
	pub fn from_rgba(image_buffer: RgbaImage) -> Self {
		let dim = image_buffer.dimensions();
		Self {
			color: true,
			dim: [dim.0, dim.1],
			data: image_buffer.into_vec(),
		}
	}

	pub fn from_luma(image_buffer: LumaImage) -> Self {
		let dim = image_buffer.dimensions();
		Self {
			color: false,
			dim: [dim.0, dim.1],
			data: image_buffer.into_vec(),
		}
	}

	pub fn luma_filled(dim: [u32; 2], value: [u8; 4]) -> Self {
		Self {
			color: false,
			dim,
			data: value
				.into_iter()
				.cycle()
				.take((dim[0] * dim[1]) as usize)
				.collect(),
		}
	}

	pub fn preset_rgb565() -> Self {
		let image = ImageBuffer::from_fn(1024, 64, |x, y| {
			image::Rgba::from([
				(x / 32) as u8 * 8,
				y as u8 * 4,
				(x % 32) as u8 * 8,
				255,
			])
		});
		Self::from_rgba(image)
	}

	pub fn load(path: &str, color: bool) -> Self {
		if color {
			Self::from_rgba(image::open(path).unwrap().into_rgba8())
		} else {
			Self::from_luma(image::open(path).unwrap().into_luma8())
		}
	}

	pub fn save(&self, path: &str) {
		if self.color {
			RgbaImage::from_vec(self.dim[0], self.dim[1], self.data.clone())
				.unwrap()
				.save(path)
				.unwrap();
		} else {
			LumaImage::from_vec(self.dim[0], self.dim[1], self.data.clone())
				.unwrap()
				.save(path)
				.unwrap();
		}
	}
}

pub fn rgb_to_16uv(rgb: [u8; 3]) -> [f32; 2] {
	let xr = (rgb[0] / 8) as f32 / 32.0;
	let xb = rgb[2] as f32 / 256.0 / 32.0;
	let xg = rgb[1] as f32 / 4.0;
	[xr + xb, xg]
}
