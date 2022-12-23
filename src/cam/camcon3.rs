use winit::event::{ElementState, MouseButton, WindowEvent, VirtualKeyCode, KeyboardInput};

use crate::{V2, V3, V4, M4};

// 3d camera controller
pub struct Camcon {
	pos: V3,
	transform: M4,

	control_state: ControlState,
}

#[derive(Default)]
struct ControlState {
	pub move_button: bool,
	pub prev_cursor_pos: Option<V2>,
}

impl Camcon {
	pub fn new(pos: V3) -> Self {
		Self {
			pos,
			transform: M4::identity(),

			control_state: Default::default(),
		}
	}

	pub fn get_camera(&self) -> M4 {
		self.transform.prepend_translation(&self.pos)
	}

	pub fn go(&mut self, dist: f32) {
		if let Some(inv) = self.transform.try_inverse() {
			let z_view: V4 = inv * V4::new(0.0, 0.0, 1.0, 0.0);
			if let Some(x) = V3::from_homogeneous(z_view) { self.pos += x * dist; }
			else {
				eprintln!("bad {}", dist);
			}
		}
	}

	pub fn rotate_view(&mut self, mut dx: V2) {
		dx *= 0.01;
		let rot = M4::from_euler_angles(-dx[1], dx[0], 0f32);
		self.transform = rot * self.transform;
	}

	pub fn process_event(&mut self, event: &WindowEvent) -> bool {
		let mut result = false;
		match event {
			WindowEvent::CursorMoved { position, .. } => {
				let pos = V2::new(position.x as f32, position.y as f32);
				if !self.control_state.move_button {
					self.control_state.prev_cursor_pos = None;
					return false;
				}
				if let Some(prev_pos) =
					self.control_state.prev_cursor_pos.take()
				{
					self.rotate_view(pos - prev_pos);
					result = true;
				}
				self.control_state.prev_cursor_pos = Some(pos);
			}
			WindowEvent::KeyboardInput {
				input: KeyboardInput {
					state: ElementState::Pressed,
					virtual_keycode: Some(vkc),
					..
				},
				..
			} => {
				result = true;
				match vkc {
					VirtualKeyCode::W => self.go(0.1),
					VirtualKeyCode::S => self.go(-0.1),
					_ => result = false,
				}
			}
			WindowEvent::MouseInput { state, button, .. } => {
				if *button == MouseButton::Middle {
					self.control_state.move_button =
						*state == ElementState::Pressed;
				}
			}
			_ => {}
		}
		result
	}
}
