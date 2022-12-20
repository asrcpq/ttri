use ttri::reexport::winit::{
	event_loop::{ControlFlow, EventLoopBuilder},
	event::{Event, WindowEvent},
};

use ttri::camcon::Camcon;
use ttri::teximg::Teximg;
use ttri::renderer::Renderer;
use ttri_model::cmodel::{Model, Face};
use ttri_model::draw::v2p4;

type V2 = rust_stddep::nalgebra::Vector2<f32>;

fn main() {
	let el = EventLoopBuilder::<()>::with_user_event().build();
	let mut rdr = Renderer::new(&el);
	let tex2 = Teximg::preset_rgb565();
	let tex1 = Teximg::load("/tmp/t/gradient.png", true);
	let mut camcon = Camcon::new([640, 480]);
	camcon.fit_inner(V2::new(0.0, 0.0), V2::new(2.0, 2.0));
	let mut _mh = Vec::new();
	rdr.upload_tex(tex1, 0);
	rdr.upload_tex(tex2, 1);
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => {
			camcon.process_event(&e);
			match e {
				WindowEvent::CloseRequested => {
					*ctrl = ControlFlow::Exit;
				}
				WindowEvent::Resized(_) => {
					rdr.damage();
					camcon.resize(rdr.get_size());
				}
				_ => {}
			}
		}
		Event::MainEventsCleared => {
			let vs = vec![
				v2p4(V2::new(0.0, 0.0), 0.0),
				v2p4(V2::new(0.0, 1.0), 0.0),
				v2p4(V2::new(1.0, 0.0), 0.0),
				v2p4(V2::new(1.0, 1.0), 0.0),
				v2p4(V2::new(1.0, 2.0), 0.0),
				v2p4(V2::new(2.0, 1.0), 0.0),
				v2p4(V2::new(2.0, 2.0), 0.0),
			];
			let uvs = vec![
				V2::new(0.0, 0.0).into(),
				V2::new(0.0, 1.0).into(),
				V2::new(1.0, 0.0).into(),
				V2::new(1.0, 1.0).into(),
			];
			let faces = vec![
				Face {
					vid: [0, 1, 2],
					uvid: [0, 1, 2],
					layer: 0,
					color: [0f32; 4],
				},
				Face {
					vid: [3, 1, 2],
					uvid: [3, 1, 2],
					layer: 0,
					color: [0f32; 4],
				},
				Face {
					vid: [3, 4, 5],
					uvid: [0, 1, 2],
					layer: 1,
					color: [0f32; 4],
				},
				Face {
					vid: [6, 4, 5],
					uvid: [3, 1, 2],
					layer: 1,
					color: [0f32; 4],
				},
			];
			let model = Model {vs, uvs, faces};
			_mh = vec![rdr.insert_model(&model)];
			rdr.render(camcon.get_camera());
			*ctrl = ControlFlow::Wait;
		}
		_ => {},
	})
}

