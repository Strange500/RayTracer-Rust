//! GPU-accelerated ray tracing using wgpu compute shaders.
//!
//! This module provides GPU-based rendering as an alternative to CPU rendering.
//! It uses wgpu for cross-platform GPU access and compute shaders written in WGSL.

#[cfg(feature = "gpu")]
use wgpu;
#[cfg(feature = "gpu")]
use wgpu::util::DeviceExt;
#[cfg(feature = "gpu")]
use bytemuck;
#[cfg(feature = "gpu")]
use pollster;

use crate::imgcomparator::Image;
use crate::raytracer::config::Config;

#[cfg(feature = "gpu")]
use bytemuck::{Pod, Zeroable};

/// GPU-compatible representation of a sphere
#[cfg(feature = "gpu")]
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GPUSphere {
    center: [f32; 3],
    radius: f32,
    diffuse: [f32; 3],
    _padding1: f32,
    specular: [f32; 3],
    shininess: f32,
}

/// GPU-compatible representation of a light
#[cfg(feature = "gpu")]
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GPULight {
    position_or_direction: [f32; 3],
    light_type: u32, // 0 = point, 1 = directional
    color: [f32; 3],
    _padding: u32,
}

/// GPU-compatible camera parameters
#[cfg(feature = "gpu")]
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GPUCamera {
    position: [f32; 3],
    _padding1: f32,
    direction: [f32; 3],
    _padding2: f32,
    up: [f32; 3],
    fov: f32,
}

/// GPU-compatible scene parameters
#[cfg(feature = "gpu")]
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GPUSceneParams {
    width: u32,
    height: u32,
    max_depth: u32,
    sphere_count: u32,
    light_count: u32,
    _padding: [u32; 3],
    ambient: [f32; 3],
    _padding2: f32,
}

#[cfg(feature = "gpu")]
pub struct GPURenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

#[cfg(feature = "gpu")]
impl GPURenderer {
    /// Initialize GPU renderer and create compute pipeline
    pub async fn new_async() -> Result<Self, String> {
        // Request GPU adapter
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find suitable GPU adapter")?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Ray Tracing Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to request device: {}", e))?;

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracing Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ray_tracing.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Bind Group Layout"),
            entries: &[
                // Scene parameters
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
                // Camera
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
                // Spheres
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
                // Lights
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
                // Output image
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracing Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            pipeline,
        })
    }

    /// Synchronous wrapper for GPU initialization
    pub fn new() -> Result<Self, String> {
        pollster::block_on(Self::new_async())
    }

    /// Render scene using GPU
    pub fn render(&self, config: &Config) -> Result<Image, String> {
        // Convert scene data to GPU format
        let spheres = self.extract_spheres(config);
        let lights = self.extract_lights(config);
        
        if spheres.is_empty() {
            return Err("GPU rendering currently only supports spheres".to_string());
        }

        let scene_params = GPUSceneParams {
            width: config.width,
            height: config.height,
            max_depth: config.maxdepth,
            sphere_count: spheres.len() as u32,
            light_count: lights.len() as u32,
            _padding: [0; 3],
            ambient: [config.ambient.x, config.ambient.y, config.ambient.z],
            _padding2: 0.0,
        };

        let camera_params = GPUCamera {
            position: [
                config.camera.position.x,
                config.camera.position.y,
                config.camera.position.z,
            ],
            _padding1: 0.0,
            direction: [
                config.camera.direction().x,
                config.camera.direction().y,
                config.camera.direction().z,
            ],
            _padding2: 0.0,
            up: [config.camera.up.x, config.camera.up.y, config.camera.up.z],
            fov: config.camera.fov,
        };

        // Create buffers
        let scene_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Params Buffer"),
            contents: bytemuck::cast_slice(&[scene_params]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let camera_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_params]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let spheres_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spheres Buffer"),
            contents: bytemuck::cast_slice(&spheres),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let lights_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: bytemuck::cast_slice(&lights),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer_size = (config.width * config.height * 4) as u64;
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

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Bind Group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: scene_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Execute compute shader
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Ray Tracing Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch work groups (8x8 threads per workgroup)
            let workgroup_size_x = 8;
            let workgroup_size_y = 8;
            let num_workgroups_x = (config.width + workgroup_size_x - 1) / workgroup_size_x;
            let num_workgroups_y = (config.height + workgroup_size_y - 1) / workgroup_size_y;
            
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }

        // Copy output to staging buffer
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            output_buffer_size,
        );

        self.queue.submit(Some(encoder.finish()));

        // Read back results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        
        self.device.poll(wgpu::Maintain::Wait);
        
        receiver.recv().unwrap().map_err(|e| format!("Failed to map buffer: {:?}", e))?;

        let data = buffer_slice.get_mapped_range();
        let pixels: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(Image::new(config.width, config.height, pixels))
    }

    /// Extract sphere data from scene
    fn extract_spheres(&self, config: &Config) -> Vec<GPUSphere> {
        use crate::raytracer::config::shape::Shape;
        
        config
            .get_scene_objects()
            .iter()
            .filter_map(|obj| match obj {
                Shape::Sphere {
                    center,
                    radius,
                    diffuse_color,
                    specular_color,
                    shininess,
                    ..
                } => Some(GPUSphere {
                    center: [center.x, center.y, center.z],
                    radius: *radius,
                    diffuse: [diffuse_color.x, diffuse_color.y, diffuse_color.z],
                    _padding1: 0.0,
                    specular: [specular_color.x, specular_color.y, specular_color.z],
                    shininess: *shininess,
                }),
                _ => None,
            })
            .collect()
    }

    /// Extract light data from scene
    fn extract_lights(&self, config: &Config) -> Vec<GPULight> {
        use crate::raytracer::config::light::Light;
        
        config
            .get_lights()
            .iter()
            .map(|light| match light {
                Light::Point { position, color } => GPULight {
                    position_or_direction: [position.x, position.y, position.z],
                    light_type: 0,
                    color: [color.x, color.y, color.z],
                    _padding: 0,
                },
                Light::Directional { direction, color } => GPULight {
                    position_or_direction: [direction.x, direction.y, direction.z],
                    light_type: 1,
                    color: [color.x, color.y, color.z],
                    _padding: 0,
                },
            })
            .collect()
    }
}

// Stub implementation when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub struct GPURenderer;

#[cfg(not(feature = "gpu"))]
impl GPURenderer {
    pub fn new() -> Result<Self, String> {
        Err("GPU support not compiled. Enable the 'gpu' feature flag.".to_string())
    }

    pub fn render(&self, _config: &Config) -> Result<Image, String> {
        Err("GPU support not compiled".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "gpu")]
    fn test_gpu_renderer_compiles() {
        // This test just ensures the GPU renderer code compiles correctly.
        // We can't actually run GPU code in CI without a GPU, so we just
        // check that the types and functions exist.
        
        // Try to create a GPU renderer - it may fail if no GPU is available,
        // but that's okay for this test
        let _ = GPURenderer::new();
    }

    #[test]
    #[cfg(not(feature = "gpu"))]
    fn test_gpu_renderer_stub_fails_correctly() {
        // When GPU feature is disabled, renderer should return an error
        let result = GPURenderer::new();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.contains("GPU support not compiled"));
    }
}
