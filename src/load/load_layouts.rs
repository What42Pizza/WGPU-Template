use crate::prelude::*;



pub fn load_render_layouts(render_context: &RenderContextData) -> Result<RenderLayouts> {
	
	let bind_0_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("bind_0_layout"),
		entries: &[
			
			// basics
			wgpu::BindGroupLayoutEntry { // camera: proj_view_mat, inv_proj_mat, view_mat
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // models: sampler
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			
			// shadow_caster
			wgpu::BindGroupLayoutEntry { // shadow_caster: proj_mat
				binding: 2,
				visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // shadowmap: tex_view
				binding: 3,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Depth,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // shadowmap: sampler
				binding: 4,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Comparison),
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // shadowmap: debug_sampler
				binding: 5,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			
			// skybox
			wgpu::BindGroupLayoutEntry { // skybox: tex_view
				binding: 6,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::Cube,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry { // skybox: sampler
				binding: 7,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler (wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			
		]
	});
	
	
	let (
		shadow_caster_pipeline,
	) = load_shadow_caster_layouts(render_context, &bind_0_layout)?;
	
	let (
		models_pipeline,
		models_bind_1_layout,
	) = load_models_layouts(render_context, &bind_0_layout)?;
	
	let (
		skybox_pipeline,
	) = load_skybox_layouts(render_context, &bind_0_layout)?;
	
	
	Ok(RenderLayouts {
		
		bind_0_layout,
		
		shadow_caster_pipeline,
		
		models_pipeline,
		models_bind_1_layout,
		
		skybox_pipeline,
		
	})
}





pub fn load_shadow_caster_layouts(render_context: &RenderContextData, bind_0_layout: &wgpu::BindGroupLayout) -> Result<(
	wgpu::RenderPipeline,
)> {
	
	
	let shadow_caster_shader_path = utils::get_program_file_path("shaders/shadow caster.wgsl");
	let shadow_caster_shader_source = fs::read_to_string(&shadow_caster_shader_path).add_path_to_error(&shadow_caster_shader_path)?;
	let shadow_caster_shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("shadow_caster_shader_module"),
		source: wgpu::ShaderSource::Wgsl(shadow_caster_shader_source.into()),
	});
	
	
	let shadow_caster_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("shadow_caster_pipeline_layout"),
		bind_group_layouts: &[
			&bind_0_layout,
		],
		push_constant_ranges: &[],
	});
	let shadow_caster_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("shadow_caster_pipeline"),
		layout: Some(&shadow_caster_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shadow_caster_shader,
			entry_point: "vs_main",
			buffers: &[
				BasicVertexData::get_layout(),
				ExtendedVertexData::get_layout(),
				RawInstanceData::get_layout()
			],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: None,
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::LessEqual,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState {
				constant: 2, /// corresponds to bilinear filtering
				slope_scale: 2.0,
				clamp: 0.0,
			},
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0u64,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	
	
	Ok((
		shadow_caster_pipeline,
	))
}





pub fn load_models_layouts(render_context: &RenderContextData, bind_0_layout: &wgpu::BindGroupLayout) -> Result<(
	wgpu::RenderPipeline,
	wgpu::BindGroupLayout,
)> {
	
	
	let models_shader_path = utils::get_program_file_path("shaders/models.wgsl");
	let models_shader_source = fs::read_to_string(&models_shader_path).add_path_to_error(&models_shader_path)?;
	let models_shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("models_shader_module"),
		source: wgpu::ShaderSource::Wgsl(models_shader_source.into()),
	});
	
	
	let models_bind_1_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some("models_bind_1_layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry { // models: tex_iew
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
		],
	});
	
	
	let models_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("models_render_pipeline_layout"),
		bind_group_layouts: &[
			&bind_0_layout,
			&models_bind_1_layout,
		],
		push_constant_ranges: &[],
	});
	let models_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("models_render_pipeline"),
		layout: Some(&models_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &models_shader,
			entry_point: "vs_main",
			buffers: &[
				BasicVertexData::get_layout(),
				ExtendedVertexData::get_layout(),
				RawInstanceData::get_layout(),
			],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: &models_shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
				format: render_context.surface_config.format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			})],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0u64,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	
	
	Ok((
		models_pipeline,
		models_bind_1_layout,
	))
}





pub fn load_skybox_layouts(render_context: &RenderContextData, bind_0_layout: &wgpu::BindGroupLayout) -> Result<(
	wgpu::RenderPipeline,
)> {
	
	
	let shader_path = utils::get_program_file_path("shaders/skybox.wgsl");
	let shader_source = fs::read_to_string(&shader_path).add_path_to_error(&shader_path)?;
	let shader = render_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("skybox_shader_module"),
		source: wgpu::ShaderSource::Wgsl(shader_source.into()),
	});
	
	
	let skybox_pipeline_layout = render_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("skybox_pipeline_layout"),
		bind_group_layouts: &[
			&bind_0_layout,
		],
		push_constant_ranges: &[],
	});
	let skybox_pipeline = render_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("skybox_pipeline"),
		layout: Some(&skybox_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
				format: render_context.surface_config.format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			})],
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Cw,
			cull_mode: Some(wgpu::Face::Back), // todo: change this to None and draw 2 smaller triangles? easiest way is to generate coords (-1, -1), (-1, 1), (1, -1), (1, 1), but that gives one ccw and one cw. There's probably not anything wrong with one large tri though
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::LessEqual,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0u64,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	});
	
	
	Ok((
		skybox_pipeline,
	))
}
