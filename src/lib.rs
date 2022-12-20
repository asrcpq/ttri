pub mod camcon;
pub mod model;
pub mod renderer;
pub mod teximg;

mod base;
mod camera;
mod helper;
mod rmod;
mod texman;
mod vertex;


pub type V2 = rust_stddep::nalgebra::Vector2<f32>;
pub type M4 = rust_stddep::nalgebra::Matrix4<f32>;

pub mod reexport {
	pub use winit;
}

pub fn printnow(s: &str) {
	let dt = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap();
	eprintln!("{}: {}", s, dt.as_secs_f64());
}
