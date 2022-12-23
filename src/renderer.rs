use rust_stddep::nalgebra;
use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::image::ImageAccess;
use vulkano::instance::debug::{
	DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
	DebugUtilsMessengerCreateInfo,
};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::SwapchainPresentInfo;
use vulkano::swapchain::{
	self, AcquireError, SwapchainCreateInfo, SwapchainCreationError,
};
use vulkano::sync::{self, GpuFuture};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::base::Base;
use crate::camera::Camera;
use crate::helper::*;
use crate::model::cmodel::Model;
use crate::model::model_ref::ModelRef;
use crate::rmod::Rmod;
use crate::teximg::Teximg;
use crate::M4;

pub struct Renderer {
	base: Base,
	rmod: Rmod,
	viewport: Viewport,
	dirty: bool,
	future: Option<VkwFuture>,
	_debug_callback: Option<DebugUtilsMessenger>,
}

// texman
impl Renderer {
	pub fn upload_tex(&mut self, image: Teximg, id: i32) {
		let mut builder = AutoCommandBufferBuilder::primary(
			&self.base.comalloc,
			self.base.queue.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		)
		.unwrap();
		self.rmod.texman.upload(
			image,
			id,
			self.base.memalloc.clone(),
			&mut builder,
		);
		let command_buffer = Box::new(builder.build().unwrap());
		let future = sync::now(self.base.device.clone())
			.then_execute(self.base.queue.clone(), command_buffer)
			.unwrap()
			.then_signal_fence_and_flush()
			.unwrap();
		self.future = Some(future.boxed());
	}

	pub fn remove_tex(&mut self, outer: i32) {
		self.rmod.texman.remove(outer);
	}
}

impl Renderer {
	pub fn new<E>(el: &EventLoopWindowTarget<E>) -> Self {
		let base = Base::new(el);
		let rmod = Rmod::new(base.clone());
		let viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [800.0, 600.0],
			depth_range: 0.0..1.0,
		};

		let mut result = Self {
			base,
			rmod,
			viewport,
			dirty: false,
			future: None,
			_debug_callback: None,
		};
		result.upload_tex(Teximg::luma_filled([1, 1], [0; 4]), -2);
		result
	}

	pub fn with_debugger(mut self) -> Self {
		unsafe {
			self._debug_callback =
				Some(get_debug_callback(self.base.instance.clone()));
		}
		self
	}

	pub fn get_window(&self) -> &Window {
		self.base
			.surface
			.object()
			.unwrap()
			.downcast_ref::<Window>()
			.unwrap()
	}

	pub fn get_size(&self) -> [u32; 2] {
		self.get_window().inner_size().into()
	}

	pub fn redraw(&mut self) {
		self.get_window().request_redraw();
	}

	pub fn damage(&mut self) {
		self.dirty = true;
	}

	pub fn insert_model(&mut self, model: &Model) -> ModelRef {
		self.rmod.modelman.insert(model, &self.rmod.texman.mapper)
	}

	pub fn render_p(&mut self, view: M4) {
		let proj = nalgebra::geometry::Perspective3::new(
			1.0,
			1.0,
			0.1,
			1000.0,
		);
		let camera = Camera {
			view: view.into(),
			proj: proj.into_inner().into(),
		};
		self.render(camera);
	}

	pub fn render_s(&mut self) {
		let [w, h]: [u32; 2] = self.get_window().inner_size().into();
		let [w, h] = [w as f32, h as f32];
		let camera = M4::new_orthographic(0., w, 0., h, 1.0, -1.0);
		self.render_o(camera);
	}

	pub fn render_o(&mut self, proj: M4) {
		let camera = Camera {
			view: M4::identity().into(),
			proj: proj.into(),
		};
		self.render(camera);
	}

	pub fn render(&mut self, camera: Camera) {
		if self.dirty {
			self.create_swapchain();
			self.dirty = false;
		}
		let (image_num, _, acquire_future) = match swapchain::acquire_next_image(
			self.base.swapchain.clone(),
			None,
		) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				self.dirty = true;
				return;
			}
			Err(e) => panic!("{:?}", e),
		};

		let mut builder = AutoCommandBufferBuilder::primary(
			&self.base.comalloc,
			self.base.queue.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		)
		.unwrap();
		if let Some(future) = self.future.take() {
			drop(future);
		}
		self.rmod.build_command(
			&mut builder,
			image_num as usize,
			camera,
			self.viewport.clone(),
		);
		let command_buffer = Box::new(builder.build().unwrap());

		let future = sync::now(self.base.device.clone())
			.join(acquire_future)
			.then_execute(self.base.queue.clone(), command_buffer)
			.unwrap()
			.then_swapchain_present(
				self.base.queue.clone(),
				SwapchainPresentInfo::swapchain_image_index(
					self.base.swapchain.clone(),
					image_num,
				),
			)
			.then_signal_fence_and_flush()
			.unwrap();
		self.future = Some(future.boxed());
	}

	fn create_swapchain(&mut self) {
		let dimensions: [u32; 2] = self.get_window().inner_size().into();
		let swapchain = self.base.swapchain.clone();
		let (new_swapchain, new_images) =
			match swapchain.recreate(SwapchainCreateInfo {
				image_extent: dimensions,
				..swapchain.create_info()
			}) {
				Ok(r) => r,
				Err(SwapchainCreationError::ImageExtentNotSupported {
					..
				}) => {
					eprintln!("Error: unsupported dimensions");
					return;
				}
				Err(e) => {
					panic!("Failed to recreate swapchain: {:?}", e)
				}
			};
		self.base.swapchain = new_swapchain;

		let dimensions = new_images[0].dimensions().width_height();
		self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
		self.rmod.update_framebuffers(&new_images);
		self.base.images = new_images;
	}
}

unsafe fn get_debug_callback(instance: VkwInstance) -> DebugUtilsMessenger {
	DebugUtilsMessenger::new(
		instance,
		DebugUtilsMessengerCreateInfo {
			message_severity: DebugUtilsMessageSeverity {
				error: true,
				warning: true,
				information: true,
				verbose: true,
				..DebugUtilsMessageSeverity::empty()
			},
			message_type: DebugUtilsMessageType {
				general: true,
				validation: true,
				performance: true,
				..DebugUtilsMessageType::empty()
			},
			..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
				let severity = if msg.severity.error {
					"error"
				} else if msg.severity.warning {
					"warning"
				} else if msg.severity.information {
					"information"
				} else if msg.severity.verbose {
					"verbose"
				} else {
					panic!("no-impl");
				};

				let ty = if msg.ty.general {
					"general"
				} else if msg.ty.validation {
					"validation"
				} else if msg.ty.performance {
					"performance"
				} else {
					panic!("no-impl");
				};

				println!(
					"{} {} {}: {}",
					msg.layer_prefix.unwrap_or("unknown"),
					ty,
					severity,
					msg.description
				);
			}))
		},
	)
	.unwrap()
}
