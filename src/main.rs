// Started:      24/04/18
// Last updated: 24/05/02

// Learn Wgpu website: https://sotrh.github.io/learn-wgpu/
// Learn Wgpu repo: https://github.com/sotrh/learn-wgpu
// Skybox source: https://opengameart.org/content/clouds-skybox-1



#![feature(duration_constants)]
#![feature(let_chains)]

#![allow(unused)]
#![warn(unused_must_use)]

#![allow(clippy::new_without_default)]
#![warn(clippy::todo)]
#![deny(clippy::unwrap_used, clippy::panic)]

// enable to see uses
//#[warn(clippy::expect_used)]



pub mod load;
pub mod update;
pub mod render;
pub mod data;
pub mod materials_storage_utils;
pub mod utils;

pub mod prelude {
	pub use crate::{*, data::*};
	pub use std::{
		fs,
		collections::HashMap,
		path::{Path, PathBuf},
		time::{Duration, Instant}
	};
	pub use std::result::Result as StdResult;
	pub use log::{info, warn, debug, error};
	pub use anyhow::*;
}

use crate::prelude::*;
use std::{env, mem, thread};
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{KeyEvent, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop},
	keyboard::PhysicalKey,
	platform::pump_events::EventLoopExtPumpEvents,
	window::{Window, WindowId}
};



fn main() -> Result<()> {
	let start_time = Instant::now();
	
	if env::var("RUST_LOG").is_err() {
		env::set_var("RUST_LOG", "warn");
	}
	env_logger::init();
	
	// There's kinda a catch-22 here where A: we need the window to be available before
	// we create the application struct, B: we need the application struct in order to
	// start the event loop, and C: we need to start the event loop to create a window.
	// So, we use EventLoopExtPumpEvents::pump_app_events to run the event loop until we
	// can get a window, then use that to create the application struct, then use that to
	// start the event loop
	info!("Running initialization event_loop...");
	let mut event_loop = EventLoop::new()?;
	let mut init_data = InitData::default();
	let window = loop {
		event_loop.pump_app_events(None, &mut init_data);
		if let Some(window) = mem::take(&mut init_data.window) {
			break window;
		}
	};
	
	info!("Done, starting main event_loop...");
	let mut program_data = load::load_program_data(start_time, &window)?;
	event_loop.run_app(&mut program_data)?;
	
	Ok(())
}





// the entire purpose of this is to get a usable window

#[derive(Default)]
pub struct InitData {
	pub window: Option<Window>,
}

impl ApplicationHandler for InitData {
	
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_none() {
			let window_attributes = Window::default_attributes()
				.with_title("WGPU Testing")
				.with_inner_size(PhysicalSize::new(1280, 720));
			let window = event_loop.create_window(window_attributes).expect("Could not init window.");
			window.request_redraw();
			self.window = Some(window);
		}
	}
	
	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		_event: WindowEvent,
	) {}
	
	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}










impl<'a> ApplicationHandler for ProgramData<'a> {
	
	
	
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		warn!("Resumed, idk what to do here");
	}
	
	
	
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		let program_data = self;
		
		match event {
			
			WindowEvent::CloseRequested => {
				event_loop.exit();
			},
			
			WindowEvent::KeyboardInput {
				event: KeyEvent {
					physical_key: PhysicalKey::Code (key),
					state,
					..
				},
				..
			} => {
				program_data.pressed_keys.insert(key, state.is_pressed());
			},
			
			WindowEvent::RedrawRequested => {
				let result = redraw_requested(program_data, event_loop);
				if let Err(err) = result {
					error!("Fatal error while processing frame: {err}");
					event_loop.exit();
				}
			},
			
			WindowEvent::Resized (new_size) => {
				resize(program_data, new_size).expect("Failed to resize the window");
			}
			
			_ => (),
		}
	}
	
	
	
	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		let program_data = self;
		program_data.window.request_redraw();
	}
	
}



pub fn resize(program_data: &mut ProgramData, new_size: PhysicalSize<u32>) -> Result<()> {
	let render_context = &mut program_data.render_context;
	render_context.surface_size = new_size;
	render_context.aspect_ratio = new_size.width as f32 / new_size.height as f32;
	render_context.surface_config.width = new_size.width;
	render_context.surface_config.height = new_size.height;
	if new_size.width == 0 || new_size.height == 0 {return Ok(());}
	render_context.drawable_surface.configure(&render_context.device, &render_context.surface_config);
	program_data.render_assets.depth = load::load_depth_render_data(render_context)?;
	Ok(())
}





pub fn redraw_requested(program_data: &mut ProgramData, event_loop: &ActiveEventLoop) -> Result<()> {
	
	
	let frame_start_time = Instant::now();
	
	let dt = program_data.step_dt();
	update::update(program_data, dt)?;
	
	
	// make sure to only render when the window is visible
	let size = program_data.render_context.surface_size;
	if size.width > 0 && size.height > 0 {
		
		
		let surface_output_result = program_data.render_context.drawable_surface.get_current_texture();
		let surface_output = match surface_output_result {
			StdResult::Ok(v) => v,
			StdResult::Err(wgpu::SurfaceError::Lost) => {
				warn!("Surface was lost, attempting to resize...");
				resize(program_data, program_data.render_context.surface_size).context("Failed to resize window.")?;
				program_data.render_context.drawable_surface.get_current_texture()?
			}
			StdResult::Err(wgpu::SurfaceError::Outdated) => {
				warn!("Surface is outdated, attempting to resize...");
				resize(program_data, program_data.render_context.surface_size).context("Failed to resize window.")?;
				program_data.render_context.drawable_surface.get_current_texture()?
			}
			StdResult::Err(wgpu::SurfaceError::OutOfMemory) => {
				warn!("OutOfMemory error while rendering, exiting process.");
				event_loop.exit();
				return Ok(());
			}
			StdResult::Err(err) => return Err(err.into()),
		};
		
		render::render(&surface_output, program_data);
		
		
		let frame_time = frame_start_time.elapsed();
		if frame_time < program_data.min_frame_time {
			let sleep_time = program_data.min_frame_time - frame_time;
			thread::sleep(sleep_time);
		}
		
		let fps_counter_output = program_data.fps_counter.step(frame_start_time.elapsed());
		if let Some((average_fps, average_frame_time)) = fps_counter_output {
			info!("FPS: {average_fps}  (avg frame time: {average_frame_time:?})");
		}
		
		
		program_data.window.pre_present_notify();
		surface_output.present();
		
		
	}
	
	
	Ok(())
}
