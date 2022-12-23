use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct Camera {
	pub view: [[f32; 4]; 4],
	pub proj: [[f32; 4]; 4],
}
