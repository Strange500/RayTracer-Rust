use crate::imgcomparator::Image;
use crate::raytracer::config::light::Light;
use crate::raytracer::config::shape::Shape;
use crate::raytracer::config::Config;
use wgpu::util::DeviceExt;

// GPU buffer structures matching the WGSL shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuCamera {
    position: [f32; 3],
    _padding1: f32,
    direction: [f32; 3],
    _padding2: f32,
    up: [f32; 3],
    fov: f32,
    width: u32,
    height: u32,
    _padding3: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSceneData {
    ambient: [f32; 3],
    maxdepth: u32,
    num_spheres: u32,
    num_planes: u32,
    num_triangles: u32,
    num_lights: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSphere {
    center: [f32; 3],
    radius: f32,
    diffuse_color: [f32; 3],
    _padding1: f32,
    specular_color: [f32; 3],
    shininess: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuPlane {
    point: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    diffuse_color: [f32; 3],
    _padding3: f32,
    specular_color: [f32; 3],
    shininess: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuTriangle {
    v0: [f32; 3],
    _padding1: f32,
    v1: [f32; 3],
    _padding2: f32,
    v2: [f32; 3],
    _padding3: f32,
    diffuse_color: [f32; 3],
    _padding4: f32,
    specular_color: [f32; 3],
    shininess: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuLight {
    position_or_direction: [f32; 3],
    light_type: u32, // 0 = point, 1 = directional
    color: [f32; 3],
    _padding: f32,
}

pub struct GpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuBackend {
    pub fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "Failed to find a suitable GPU adapter".to_string())?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Ray Tracer Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .map_err(|e| format!("Failed to create device: {}", e))?;

        Ok(GpuBackend { device, queue })
    }

    pub fn render(&self, config: &Config) -> Result<Image, String> {
        let width = config.width as usize;
        let height = config.height as usize;

        // Prepare scene data
        let (spheres, planes, triangles, lights) = self.prepare_scene_data(config);

        let camera_data = GpuCamera {
            position: config.camera.position.to_array(),
            _padding1: 0.0,
            direction: config.camera.direction().to_array(),
            _padding2: 0.0,
            up: config.camera.up.to_array(),
            fov: config.camera.fov,
            width: config.width,
            height: config.height,
            _padding3: [0, 0],
        };

        let scene_data = GpuSceneData {
            ambient: config.ambient.to_array(),
            maxdepth: config.maxdepth,
            num_spheres: spheres.len() as u32,
            num_planes: planes.len() as u32,
            num_triangles: triangles.len() as u32,
            num_lights: lights.len() as u32,
        };

        // Create buffers
        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let scene_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Scene Buffer"),
                contents: bytemuck::cast_slice(&[scene_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let spheres_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Spheres Buffer"),
                contents: bytemuck::cast_slice(&spheres),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let planes_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Planes Buffer"),
                contents: bytemuck::cast_slice(&planes),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let triangles_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangles Buffer"),
                contents: bytemuck::cast_slice(&triangles),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let lights_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: bytemuck::cast_slice(&lights),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let output_buffer_size = (width * height * std::mem::size_of::<u32>()) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Load shader
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Ray Tracer Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        // Create bind group layout
        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Ray Tracer Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracer Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: planes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: triangles_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute pipeline
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline =
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Ray Tracer Pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                });

        // Execute compute shader
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Ray Tracer Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracer Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            let workgroup_count_x = (width as u32 + 7) / 8;
            let workgroup_count_y = (height as u32 + 7) / 8;
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_buffer_size);

        self.queue.submit(Some(encoder.finish()));

        // Read back results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);

        receiver
            .recv()
            .map_err(|e| format!("Failed to receive buffer mapping result: {}", e))?
            .map_err(|e| format!("Failed to map buffer: {:?}", e))?;

        let data = buffer_slice.get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(Image::new(config.width, config.height, result))
    }

    fn prepare_scene_data(
        &self,
        config: &Config,
    ) -> (Vec<GpuSphere>, Vec<GpuPlane>, Vec<GpuTriangle>, Vec<GpuLight>) {
        let mut spheres = Vec::new();
        let mut planes = Vec::new();
        let mut triangles = Vec::new();

        for shape in config.get_scene_objects() {
            match shape {
                Shape::Sphere {
                    center,
                    radius,
                    diffuse_color,
                    specular_color,
                    shininess,
                } => {
                    spheres.push(GpuSphere {
                        center: center.to_array(),
                        radius: *radius,
                        diffuse_color: diffuse_color.to_array(),
                        _padding1: 0.0,
                        specular_color: specular_color.to_array(),
                        shininess: *shininess,
                    });
                }
                Shape::Plane {
                    point,
                    normal,
                    diffuse_color,
                    specular_color,
                    shininess,
                } => {
                    planes.push(GpuPlane {
                        point: point.to_array(),
                        _padding1: 0.0,
                        normal: normal.to_array(),
                        _padding2: 0.0,
                        diffuse_color: diffuse_color.to_array(),
                        _padding3: 0.0,
                        specular_color: specular_color.to_array(),
                        shininess: *shininess,
                    });
                }
                Shape::Triangle {
                    v0,
                    v1,
                    v2,
                    diffuse_color,
                    specular_color,
                    shininess,
                } => {
                    triangles.push(GpuTriangle {
                        v0: v0.to_array(),
                        _padding1: 0.0,
                        v1: v1.to_array(),
                        _padding2: 0.0,
                        v2: v2.to_array(),
                        _padding3: 0.0,
                        diffuse_color: diffuse_color.to_array(),
                        _padding4: 0.0,
                        specular_color: specular_color.to_array(),
                        shininess: *shininess,
                    });
                }
            }
        }

        // Ensure at least one element in each buffer (GPU requirement)
        if spheres.is_empty() {
            spheres.push(GpuSphere {
                center: [0.0, 0.0, 0.0],
                radius: 0.0,
                diffuse_color: [0.0, 0.0, 0.0],
                _padding1: 0.0,
                specular_color: [0.0, 0.0, 0.0],
                shininess: 0.0,
            });
        }
        if planes.is_empty() {
            planes.push(GpuPlane {
                point: [0.0, 0.0, 0.0],
                _padding1: 0.0,
                normal: [0.0, 1.0, 0.0],
                _padding2: 0.0,
                diffuse_color: [0.0, 0.0, 0.0],
                _padding3: 0.0,
                specular_color: [0.0, 0.0, 0.0],
                shininess: 0.0,
            });
        }
        if triangles.is_empty() {
            triangles.push(GpuTriangle {
                v0: [0.0, 0.0, 0.0],
                _padding1: 0.0,
                v1: [0.0, 0.0, 0.0],
                _padding2: 0.0,
                v2: [0.0, 0.0, 0.0],
                _padding3: 0.0,
                diffuse_color: [0.0, 0.0, 0.0],
                _padding4: 0.0,
                specular_color: [0.0, 0.0, 0.0],
                shininess: 0.0,
            });
        }

        let mut lights = Vec::new();
        for light in config.get_lights() {
            match light {
                Light::Point { position, color } => {
                    lights.push(GpuLight {
                        position_or_direction: position.to_array(),
                        light_type: 0,
                        color: color.to_array(),
                        _padding: 0.0,
                    });
                }
                Light::Directional { direction, color } => {
                    lights.push(GpuLight {
                        position_or_direction: direction.to_array(),
                        light_type: 1,
                        color: color.to_array(),
                        _padding: 0.0,
                    });
                }
            }
        }

        if lights.is_empty() {
            lights.push(GpuLight {
                position_or_direction: [0.0, 0.0, 0.0],
                light_type: 0,
                color: [0.0, 0.0, 0.0],
                _padding: 0.0,
            });
        }

        (spheres, planes, triangles, lights)
    }
}
