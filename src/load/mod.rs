use crate::prelude::*;
use async_std::task::block_on;
use winit::{dpi::PhysicalPosition, window::Window};
use serde_hjson::{Map, Value};



pub mod load_layouts;
pub use load_layouts::*;
pub mod load_assets;
pub use load_assets::*;
pub mod load_bindings;
pub use load_bindings::*;





pub fn load_program_data(start_time: Instant, window: &Window) -> Result<ProgramData> {
	
	let engine_config = load_engine_config().context("Failed to load engine config.")?;
	let input = EngineInput {
		pressed_keys: HashSet::new(),
		prev_pressed_keys: HashSet::new(),
		mouse_pos: PhysicalPosition::default(),
		mouse_vel: PhysicalPosition::default(),
		capture_cursor: false,
		pressed_mouse_buttons: PressedMouseButtons::default(),
		prev_pressed_mouse_buttons: PressedMouseButtons::default(),
	};
	
	// app data
	let camera_data = CameraData::new((0., 1., 2.));
	let shadow_caster_data = ShadowCasterData::default();
	let example_model_instance_datas = load_example_model_instance_datas();
	let fps_counter = FpsCounter::new();
	let color_correction_settings = ColorCorrectionSettings::default();
	
	// render data
	let render_context = load_render_context_data(window, &engine_config)?;
	let render_layouts = load_render_layouts(&render_context)?;
	let render_assets = load_render_assets(
		&camera_data,
		&shadow_caster_data,
		&example_model_instance_datas,
		&render_context,
		engine_config.shadowmap_size,
		&color_correction_settings,
		engine_config.compress_textures,
	)?;
	let render_bindings = load_render_bindings(&render_context, &render_layouts, &render_assets)?;
	
	Ok(ProgramData {
		
		// engine data
		start_time,
		engine_config,
		input,
		
		// app data
		camera_data,
		shadow_caster_data,
		example_model_instance_datas,
		fps_counter,
		is_moving_camera: false,
		color_correction_settings,
		
		// render data
		render_context,
		render_layouts,
		render_assets,
		render_bindings,
		frame_start_instant: start_time,
		
	})
}





// HELP: to automatically update old config files from version 1 to version 2 (if / when
// a version 2 is made), then 2 to 3, and so on, you just add updater functions to this
// list
#[allow(clippy::type_complexity)]
const CONFIG_UPDATER_FUNCTIONS: &[fn(&mut Map<String, Value>) -> Result<()>] = &[
	
];
const LATEST_CONFIG_VERSION: usize = CONFIG_UPDATER_FUNCTIONS.len() + 1;

pub fn load_engine_config() -> Result<EngineConfig> {
	
	let engine_config_path = utils::get_program_file_path("engine config.hjson");
	let engine_config_result = fs::read_to_string(&engine_config_path);
	let engine_config_string = match &engine_config_result {
		StdResult::Ok(v) => &**v,
		StdResult::Err(err) => {
			warn!("Failed to read 'engine config.hjson' (full path: {engine_config_path:?}), using default values...  (error: {err})");
			include_str!("../../data/default engine config.hjson")
		}
	};
	let mut engine_config: Map<String, Value> = serde_hjson::from_str(engine_config_string).context("Failed to decode 'engine config.hjson'")?;
	
	// update config
	let config_version = read_hjson_i64(&engine_config, "config_version", LATEST_CONFIG_VERSION as i64);
	for i in config_version as usize .. LATEST_CONFIG_VERSION {
		CONFIG_UPDATER_FUNCTIONS[i - 1](&mut engine_config)?;
	}
	
	let rendering_backend_str = read_hjson_str(&engine_config, "rendering_backend", "auto");
	let rendering_backend = match &*rendering_backend_str.to_lowercase() {
		"auto" => wgpu::Backends::all(),
		"vulkan" => wgpu::Backends::VULKAN,
		"dx12" => wgpu::Backends::DX12,
		"metal" => wgpu::Backends::METAL,
		"opengl" => wgpu::Backends::GL,
		_ => {
			warn!("Unknown value for entry 'rendering_backend' in 'engine config.hjson', must be: 'auto', 'vulkan', 'dx12', 'metal', or 'opengl', defaulting to \"auto\".");
			wgpu::Backends::all()
		}
	};
	
	let present_mode_str = read_hjson_str(&engine_config, "present_mode", "auto_vsync");
	let present_mode = match &*present_mode_str.to_lowercase() {
		"auto_vsync" => wgpu::PresentMode::AutoVsync,
		"auto_no_vsync" => wgpu::PresentMode::AutoVsync,
		"fifo" => wgpu::PresentMode::Fifo,
		"fifo_relaxed" => wgpu::PresentMode::FifoRelaxed,
		"immediate" => wgpu::PresentMode::Immediate,
		"mailbox" => wgpu::PresentMode::Mailbox,
		_ => {
			warn!("Unknown value for entry 'present_mode' in 'engine config.hjson', must be: 'auto_vsync', 'auto_no_vsync', 'fifo', 'fifo_relaxed', 'immediate', or 'mailbox', defaulting to \"auto_vsync\".");
			wgpu::PresentMode::AutoVsync
		}
	};
	
	let desired_frame_latency_i64 = read_hjson_i64(&engine_config, "desired_frame_latency", 1);
	let desired_frame_latency = desired_frame_latency_i64 as u32;
	
	let min_frame_time_f64 = read_hjson_f64(&engine_config, "min_frame_time", 0.002);
	let min_frame_time = Duration::from_secs_f64(min_frame_time_f64);
	
	let shadowmap_size_i64 = read_hjson_i64(&engine_config, "shadowmap_size", 512);
	let shadowmap_size = shadowmap_size_i64 as u32;
	
	let compress_textures = read_hjson_bool(&engine_config, "compress_textures", true);
	
	Ok(EngineConfig {
		rendering_backend,
		present_mode,
		desired_frame_latency,
		min_frame_time,
		shadowmap_size,
		compress_textures,
	})
}



pub fn read_hjson_str<'a>(map: &'a Map<String, Value>, key: &'static str, default: &'static str) -> &'a str {
	let value_str = map.get(key);
	let value_str = value_str.map(|v| v.as_str().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be a string, defaulting to \"{default}\".");
		default
	}));
	value_str.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_i64(map: &Map<String, Value>, key: &'static str, default: i64) -> i64 {
	let value_str = map.get(key);
	let value_i64 = value_str.map(|v| v.as_i64().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be an int, defaulting to \"{default}\".");
		default
	}));
	value_i64.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_f64(map: &Map<String, Value>, key: &'static str, default: f64) -> f64 {
	let value_str = map.get(key);
	let value_f64 = value_str.map(|v| v.as_f64().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be a number, defaulting to \"{default}\".");
		default
	}));
	value_f64.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}

pub fn read_hjson_bool(map: &Map<String, Value>, key: &'static str, default: bool) -> bool {
	let value_str = map.get(key);
	let value_bool = value_str.map(|v| v.as_bool().unwrap_or_else(|| {
		warn!("Entry '{key}' in 'engine config.hjson' must be a bool, defaulting to \"{default}\".");
		default
	}));
	value_bool.unwrap_or_else(|| {
		warn!("Could not find entry '{key}' in 'engine config.hjson', defaulting to \"{default}\".");
		default
	})
}





// HELP: this is just random data as an example
pub fn load_example_model_instance_datas() -> Vec<InstanceData> {
	const X_LEN: usize = 100;
	const Z_LEN: usize = 100;
	let mut output = Vec::with_capacity(X_LEN * Z_LEN);
	for z in 0..Z_LEN {
		for x in 0..X_LEN {
			let pos = glam::Vec3 { x: x as f32 * 3.0, y: 0.0, z: z as f32 * 3.0 } - glam::Vec3::new(0.5, 0.0, 0.5);
			let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, pos.x / 10.0, pos.y / 10.0, pos.z / 10.0);
			output.push(InstanceData {
				pos,
				rot,
			})
		}
	}
	for z in 0..Z_LEN {
		for x in 0..X_LEN {
			let pos = glam::Vec3 { x: x as f32 * 3.0, y: 10.0, z: z as f32 * 3.0 } - glam::Vec3::new(0.5, 0.0, 0.5);
			let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, pos.x / 10.0, pos.y / 10.0, pos.z / 10.0);
			output.push(InstanceData {
				pos,
				rot,
			})
		}
	}
	output
}





pub fn load_render_context_data<'a>(window: &'a Window, engine_config: &load::EngineConfig) -> Result<RenderContextData<'a>> {
	block_on(load_render_context_data_async(window, engine_config))
}

pub async fn load_render_context_data_async<'a>(window: &'a Window, engine_config: &load::EngineConfig) -> Result<RenderContextData<'a>> {
	let surface_size = window.inner_size();
	
	// The instance is a handle to our GPU
	// Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
		backends: engine_config.rendering_backend,
		..Default::default()
	});
	
	// Handle to a presentable surface
	let surface = instance.create_surface(window).context("Failed to create drawable surface for window.")?;
	
	// Handle to a physical graphics and/or compute device
	let mut adapter = instance.request_adapter(
		&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		},
	).await;
	if adapter.is_none() {
		adapter =
			instance
			.enumerate_adapters(wgpu::Backends::all()).into_iter()
			.find(|adapter| adapter.is_surface_supported(&surface));
	}
	let Some(adapter) = adapter else {return Err(Error::msg("Unable to find suitable adapter."));};
	
	// Open connection to a graphics and/or compute device, Handle to a command queue on a device
	let (device, command_queue) = adapter.request_device(
		&wgpu::DeviceDescriptor {
			required_features: wgpu::Features::empty() | wgpu::Features::TEXTURE_COMPRESSION_BC,
			required_limits: wgpu::Limits::downlevel_defaults(),
			label: None,
		},
		None,
	).await.context("Failed to create connection to gpu.")?;
	
	let surface_caps = surface.get_capabilities(&adapter);
	let surface_format = surface_caps.formats.iter()
		.copied()
		.find(|f| f.is_srgb())
		.unwrap_or(surface_caps.formats[0]);
	let surface_config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: surface_format,
		width: surface_size.width,
		height: surface_size.height,
		present_mode: engine_config.present_mode,
		alpha_mode: surface_caps.alpha_modes[0],
		view_formats: vec![],
		desired_maximum_frame_latency: engine_config.desired_frame_latency,
	};
	surface.configure(&device, &surface_config);
	
	Ok(RenderContextData {
		window,
		drawable_surface: surface,
		device,
		command_queue,
		surface_config,
		surface_size,
		surface_format,
		aspect_ratio: surface_size.width as f32 / surface_size.height as f32,
	})
}
