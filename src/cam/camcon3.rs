use winit::event::{ElementState, MouseButton, WindowEvent, VirtualKeyCode, KeyboardInput};

use crate::{V2, V3, M4};

// 3d camera controller
pub struct Camcon {
	pos: V3,
	look: V3, // relative, norm
	up: V3, // relative, norm

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
			look: V3::new(0.0, 0.0, 1.0),
			up: V3::new(0.0, 1.0, 0.0),

			control_state: Default::default(),
		}
	}

	pub fn get_camera(&self) -> M4 {
		rust_stddep::nalgebra_glm::look_at(
			&self.pos,
			&(self.pos + self.look),
			&self.up,
		)
	}

	pub fn right(&mut self, dist: f32) {
		let right = self.look.cross(&self.up);
		self.pos += dist * right;
	}

	pub fn go(&mut self, dist: f32) {
		self.pos += dist * self.look;
	}

	pub fn rotate_view(&mut self, dx: V2) {
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
