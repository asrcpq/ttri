use rust_stddep::winit::event::{ElementState, MouseButton, WindowEvent};

use crate::{V2, V3, V4, M4};

// 3d camera controller
pub struct Camcon {
	pos: V3,

	yaw: f32,
	pitch: f32,

	control_state: ControlState,
}

#[derive(Default)]
struct ControlState {
	pub mouse_down: bool,
	pub ctrl_button: bool,
	pub shift_button: bool,
	pub prev_cursor_pos: Option<V2>,
}

impl Camcon {
	pub fn new(pos: V3) -> Self {
		Self {
			pos,
			yaw: 0.0,
			pitch: 0.0,

			control_state: Default::default(),
		}
	}

	pub fn get_camera(&self) -> M4 {
		self.get_trans().prepend_translation(&self.pos)
	}

	pub fn get_trans(&self) -> M4 {
		M4::from_euler_angles(-self.pitch, 0f32, 0f32) *
			M4::from_euler_angles(0f32, self.yaw, 0f32)
	}

	pub fn go(&mut self, mut dist: f32) {
		dist *= -0.1;
		if let Some(inv) = self.get_trans().try_inverse() {
			let z_view: V4 = inv * V4::new(0.0, 0.0, 1.0, 0.0);
			if let Some(x) = V3::from_homogeneous(z_view) { self.pos += x * dist; }
			else {
				eprintln!("bad {}", dist);
			}
		}
	}

	pub fn move_view(&mut self, mut dx: V2) {
		dx *= 0.03;
		if let Some(inv) = self.get_trans().try_inverse() {
			let x_view: V4 = inv * V4::new(1.0, 0.0, 0.0, 0.0);
			let y_view: V4 = inv * V4::new(0.0, 1.0, 0.0, 0.0);
			if let Some(x) = V3::from_homogeneous(x_view) { self.pos += x * dx[0]; }
			if let Some(x) = V3::from_homogeneous(y_view) { self.pos += x * dx[1]; }
		}
	}

	pub fn rotate_view(&mut self, mut dx: V2) {
		dx *= 0.003;
		self.yaw += dx[0];
		self.pitch += dx[1];
	}

	pub fn process_event(&mut self, event: &WindowEvent) -> bool {
		let mut result = false;
		match event {
			WindowEvent::CursorMoved { position, .. } => {
				let pos = V2::new(position.x as f32, position.y as f32);
				if !self.control_state.mouse_down {
					self.control_state.prev_cursor_pos = None;
					return false;
				}
				if let Some(prev_pos) =
					self.control_state.prev_cursor_pos.take()
				{
					let dp = pos - prev_pos;
					if self.control_state.ctrl_button {
						self.go(dp[1]);
					} else if self.control_state.shift_button {
						self.rotate_view(dp);
					} else {
						self.move_view(dp);
					}
					result = true;
				}
				self.control_state.prev_cursor_pos = Some(pos);
			}
			WindowEvent::MouseInput { state, button, .. } => {
				if *button == MouseButton::Middle {
					self.control_state.mouse_down = *state == ElementState::Pressed;
				}
			}
			WindowEvent::ModifiersChanged(state) => {
				self.control_state.ctrl_button = state.ctrl();
				self.control_state.shift_button = state.shift();
			}
			_ => {}
		}
		result
	}
}
