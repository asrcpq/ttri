use rust_stddep::winit::dpi::{LogicalSize, Size};
use rust_stddep::winit::event_loop::EventLoopWindowTarget;
use rust_stddep::winit::window::{Window, WindowBuilder};
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{
	Device, DeviceCreateInfo, DeviceExtensions, Features, QueueCreateInfo,
};
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::{
	PresentMode, Swapchain, SwapchainCreateInfo, Surface, SurfaceCreationError};
use vulkano::{Version, VulkanLibrary};

use crate::helper::*;

#[derive(Clone)]
pub struct Base {
	pub instance: VkwInstance,
	pub device: VkwDevice,
	pub queue: VkwQueue,
	pub surface: VkwSurface,
	pub swapchain: VkwSwapchain,
	pub images: VkwImages,
	pub memalloc: VkwMemAlloc,
	pub dstalloc: VkwDstAlloc,
	pub comalloc: VkwComAlloc,
}

fn winit_size(size: [u32; 2]) -> Size {
	Size::new(LogicalSize::new(size[0], size[1]))
}

pub fn required_extensions(library: &VulkanLibrary) -> InstanceExtensions {
	let ideal = InstanceExtensions {
		khr_surface: true,
		khr_xlib_surface: true,
		khr_xcb_surface: true,
		khr_wayland_surface: true,
		khr_get_physical_device_properties2: true,
		khr_get_surface_capabilities2: true,
		..InstanceExtensions::empty()
	};

	library.supported_extensions().intersection(&ideal)
}

unsafe fn winit_to_surface(
	instance: Arc<Instance>,
	window: Arc<Window>,
) -> Result<Arc<Surface>, SurfaceCreationError> {
	use rust_stddep::winit::platform::unix::WindowExtUnix;

	match (window.wayland_display(), window.wayland_surface()) {
		(Some(display), Some(surface)) => {
			Surface::from_wayland(instance, display, surface, Some(window))
		}
		_ => {
			// No wayland display found, check if we can use xlib.
			// If not, we use xcb.
			if instance.enabled_extensions().khr_xlib_surface {
				Surface::from_xlib(
					instance,
					window.xlib_display().unwrap(),
					window.xlib_window().unwrap() as _,
					Some(window),
				)
			} else {
				Surface::from_xcb(
					instance,
					window.xcb_connection().unwrap(),
					window.xlib_window().unwrap() as _,
					Some(window),
				)
			}
		}
	}
}

impl Base {
	pub fn new<E>(el: &EventLoopWindowTarget<E>) -> Self {
		let library = VulkanLibrary::new().unwrap();
		assert!(library.api_version() >= Version::V1_2);
		let required_extensions = required_extensions(&library);
		let extensions = InstanceExtensions {
			ext_debug_utils: true,
			..InstanceExtensions::empty()
		};

		let layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];
		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				enabled_extensions: required_extensions & extensions,
				enabled_layers: layers,
				..Default::default()
			},
		)
		.unwrap();
		let window = WindowBuilder::new()
			.with_inner_size(winit_size([800, 600]))
			//.with_resizable(false)
			.build(el)
			.unwrap();
		let window = Arc::new(window);
		let surface = unsafe {
			winit_to_surface(instance.clone(), window).unwrap()
		};

		let (physical_device, device, queue) =
			get_device_and_queue(&instance, surface.clone());

		let (swapchain, images) = get_swapchain_and_images(
			physical_device,
			device.clone(),
			surface.clone(),
		);
		let (memalloc, dstalloc, comalloc) = get_allocators(device.clone());
		Self {
			instance,
			device,
			queue,
			surface,
			swapchain,
			images,
			memalloc,
			dstalloc,
			comalloc,
		}
	}
}

pub fn get_allocators(
	device: VkwDevice,
) -> (VkwMemAlloc, VkwDstAlloc, VkwComAlloc) {
	let memalloc = StandardMemoryAllocator::new_default(device.clone());
	let dstalloc = StandardDescriptorSetAllocator::new(device.clone());
	let comalloc =
		StandardCommandBufferAllocator::new(device, Default::default());
	(Arc::new(memalloc), Arc::new(dstalloc), Arc::new(comalloc))
}

pub fn get_device_and_queue(
	instance: &VkwInstance,
	surface: VkwSurface,
) -> (VkwPhysicalDevice, VkwDevice, VkwQueue) {
	let device_extensions = DeviceExtensions {
		khr_swapchain: true,
		..DeviceExtensions::empty()
	};

	let features = Features {
		descriptor_binding_variable_descriptor_count: true,
		runtime_descriptor_array: true,
		..Features::empty()
	};

	let (physical_device, queue_family_index) = instance
		.enumerate_physical_devices()
		.unwrap()
		.filter(|p| p.supported_extensions().contains(&device_extensions))
		.filter(|p| p.supported_features().contains(&features))
		.filter_map(|p| {
			p.queue_family_properties()
				.iter()
				.enumerate()
				.position(|(i, q)| {
					q.queue_flags.graphics
						&& p.surface_support(i as u32, &surface)
							.unwrap_or(false)
				})
				.map(|i| (p, i as u32))
		})
		.min_by_key(|(p, _)| match p.properties().device_type {
			// TODO: detect currently using gpu
			PhysicalDeviceType::IntegratedGpu => 0,
			PhysicalDeviceType::DiscreteGpu => 1,
			PhysicalDeviceType::VirtualGpu => 2,
			PhysicalDeviceType::Cpu => 3,
			PhysicalDeviceType::Other => 4,
			_ => 5,
		})
		.expect("No suitable physical device found");

	// println!(
	//	"Using device: {} (type: {:?})",
	//	physical_device.properties().device_name,
	//	physical_device.properties().device_type,
	// );

	let (device, mut queues) = Device::new(
		physical_device.clone(),
		DeviceCreateInfo {
			enabled_extensions: device_extensions,
			enabled_features: features,
			queue_create_infos: vec![QueueCreateInfo {
				queue_family_index,
				..Default::default()
			}],

			..Default::default()
		},
	)
	.unwrap();
	let queue = queues.next().unwrap();

	(physical_device, device, queue)
}

pub fn get_swapchain_and_images(
	physical_device: VkwPhysicalDevice,
	device: VkwDevice,
	surface: VkwSurface,
) -> (VkwSwapchain, VkwImages) {
	let caps = physical_device
		.surface_capabilities(&surface, Default::default())
		.unwrap();
	let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
	let format = physical_device
		.surface_formats(&surface, Default::default())
		.unwrap()[0]
		.0;
	let format = Some(format);
	let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
	let dimensions: [u32; 2] = window.inner_size().into();

	Swapchain::new(
		device,
		surface,
		SwapchainCreateInfo {
			min_image_count: caps.min_image_count,
			image_format: format,
			image_extent: dimensions,
			image_usage: ImageUsage {
				color_attachment: true,
				..ImageUsage::empty()
			},
			composite_alpha,
			present_mode: PresentMode::Mailbox,
			..Default::default()
		},
	)
	.unwrap()
}
