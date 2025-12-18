# RayTracer-Rust

A high-performance ray tracer implementation in Rust with optional GPU acceleration.

## Features

- ✨ **CPU & GPU Rendering**: Choose between CPU (Rayon parallel) or GPU (wgpu compute shaders)
- 🎨 **Rich Scene Support**: Spheres, planes, and triangles
- 💡 **Advanced Lighting**: Point and directional lights with shadows
- 🪞 **Reflections**: Recursive reflections with configurable depth
- 🎯 **Phong Shading**: Diffuse and specular lighting with shininess control
- 🚀 **High Performance**: Parallel CPU rendering or massive GPU parallelism
- 🔄 **Automatic Fallback**: Falls back to CPU if GPU unavailable

## Quick Start

### Build

```bash
cargo build --release
```

### Run

```bash
# CPU rendering (default)
cargo run --release scene_file.scene

# GPU rendering
cargo run --release scene_file.scene --gpu
```

## Usage

See [GPU_USAGE.md](GPU_USAGE.md) for detailed usage instructions, performance tips, and troubleshooting.

## GPU Acceleration

This ray tracer supports GPU acceleration using WebGPU (wgpu). For complex scenes, GPU rendering can be 5-20x faster than CPU rendering.

**Research & Implementation**: See [GPU_ACCELERATION_RESEARCH.md](GPU_ACCELERATION_RESEARCH.md) for:
- Library evaluation (wgpu, CUDA, OpenCL)
- Architecture design
- Implementation strategy
- Performance analysis
- Trade-offs and recommendations

## Scene File Format

```
# Camera and output
size 640 480
camera 0 0 10 0 0 -1 0 1 0 45
output output.png

# Materials and lighting
diffuse 0.8 0.2 0.2
ambient 0.1 0.1 0.1
point 5 5 5 1.0 1.0 1.0
shininess 50
specular 0.5 0.5 0.5
maxdepth 3

# Objects
sphere 0 0 0 2
plane 0 -2 0 0 1 0
triangle 1 0 0 0 1 0 -1 0 0
```

## Architecture

```
src/
├── main.rs                  # Entry point with CLI handling
├── imgcomparator/           # Image utilities
│   └── mod.rs
└── raytracer/
    ├── mod.rs               # Module exports
    ├── raytracer.rs         # CPU ray tracer (Rayon)
    ├── gpu_backend.rs       # GPU backend (wgpu)
    ├── shader.wgsl          # WGSL compute shader
    └── config/              # Scene configuration
        ├── mod.rs
        ├── config_builder.rs
        ├── camera.rs
        ├── light.rs
        └── shape.rs
```

## Performance

| Backend | Simple Scene | Medium Scene | Complex Scene |
|---------|-------------|--------------|---------------|
| CPU (Rayon) | 30ms | 500ms | 5s |
| GPU (wgpu) | 100ms* | 100ms | 250ms |

*GPU has initial overhead for simple scenes

## Dependencies

- `glam`: Vector math library
- `image`: Image encoding/decoding
- `rayon`: CPU parallelization
- `wgpu`: GPU compute (WebGPU)
- `pollster`: Async runtime for GPU
- `bytemuck`: Safe type conversions for GPU

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_raytracer_tp31

# Test with custom scene
cargo run --release test.scene
```

All existing tests pass with CPU rendering. GPU rendering produces identical output when available.

## Development

### Adding Features

1. Implement in CPU renderer (`raytracer.rs`)
2. Implement in GPU shader (`shader.wgsl`)
3. Update buffer structures in `gpu_backend.rs`
4. Add tests
5. Update documentation

### Building for Production

```bash
cargo build --release
```

### Benchmarking

```bash
# Compare CPU vs GPU
time cargo run --release scene.scene
time cargo run --release scene.scene --gpu
```

## Requirements

- Rust 1.70+
- For GPU: Vulkan/Metal/DirectX 12 support
- For CPU: Multi-core processor (recommended)

## Contributing

Contributions are welcome! Please:
1. Follow existing code style
2. Add tests for new features
3. Update documentation
4. Ensure CPU and GPU implementations stay in sync

## License

See LICENSE file for details.

## Acknowledgments

- Built with Rust's excellent ecosystem
- GPU rendering powered by wgpu
- CPU parallelization via Rayon
- Math operations using glam

## Future Roadmap

- [ ] BVH acceleration structure
- [ ] Adaptive sampling
- [ ] Denoising
- [ ] Progressive rendering
- [ ] Texture mapping
- [ ] Normal mapping
- [ ] Area lights
- [ ] Global illumination
- [ ] Multi-GPU support
