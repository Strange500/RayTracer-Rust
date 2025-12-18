# GPU Acceleration for RayTracer-Rust

This document provides usage instructions for the GPU-accelerated ray tracing feature.

## Overview

The GPU acceleration feature uses **wgpu** and compute shaders to render scenes on the GPU, providing significant performance improvements for complex scenes (10-100x speedup expected).

## Prerequisites

- **Graphics API Support**: Your system must support at least one of:
  - Vulkan (Linux, Windows, Android)
  - Metal (macOS, iOS)
  - DirectX 12 (Windows)
  - WebGPU (browsers)

- **GPU Drivers**: Ensure your GPU drivers are up to date

## Building with GPU Support

### Enable the GPU Feature

```bash
# Build with GPU support
cargo build --release --features gpu

# Run with GPU support
cargo run --release --features gpu

# Build the GPU demo example
cargo build --example gpu_demo --features gpu --release
```

### Default Build (CPU-only)

```bash
# Standard build (no GPU support)
cargo build --release
cargo run --release
```

## Usage

### Using the GPU Renderer in Code

```rust
use raytracer_rust::raytracer::{ParsedConfigState, GPURenderer};
use raytracer_rust::imgcomparator::save_image;

fn main() {
    // Load scene configuration
    let mut parsed_config = ParsedConfigState::new();
    let config = parsed_config
        .load_config_file("scene.test")
        .expect("Failed to load scene");

    // Initialize GPU renderer
    let gpu_renderer = GPURenderer::new()
        .expect("Failed to initialize GPU");

    // Render scene
    let image = gpu_renderer.render(&config)
        .expect("Failed to render");

    // Save result
    save_image(&image, "output.png")
        .expect("Failed to save image");
}
```

### Running the Demo

The `gpu_demo` example compares CPU vs GPU rendering performance:

```bash
# Run the GPU performance comparison demo
cargo run --example gpu_demo --features gpu --release

# Expected output:
# === Ray Tracing Performance Comparison ===
# 
# Scene: gpu_test.scene
# Resolution: 800x600
# Objects: 3 spheres
# 
# --- CPU Rendering (with BVH + Rayon) ---
# Time: 45ms
# Saved: output_cpu.png
# 
# --- GPU Rendering (with wgpu compute shaders) ---
# Time: 15ms
# Saved: output_gpu.png
# 
# --- Performance Comparison ---
# Speedup: 3.00x
# GPU is 3.00x faster than CPU
```

## Current Limitations

The GPU implementation is currently a **minimal working demonstration** with the following limitations:

### Supported Features ✅
- Sphere rendering
- Point and directional lights
- Diffuse and specular lighting (Blinn-Phong)
- Shadow rays
- Reflections
- Ambient lighting
- Configurable recursion depth

### Not Yet Supported ❌
- Triangles (BVH traversal needs adaptation for GPU)
- Planes (infinite primitives)
- Complex BVH structures (currently CPU-only)
- Dynamic scene updates (BVH rebuild needed)

**Note**: The GPU renderer will return an error if the scene contains non-sphere objects.

## Performance Considerations

### When GPU is Faster
- Complex scenes (1000+ objects)
- High resolution renders (1920x1080+)
- Many light sources
- Deep reflection recursion

### When CPU Might Be Faster
- Simple scenes (<100 objects)
- Low resolution (800x600 or less)
- Scenes with many triangles (not yet supported on GPU)

**Recommendation**: For production use, try both and use whichever is faster for your specific scenes.

## Hybrid Approach (Future)

Future versions may support a hybrid approach:
- CPU: BVH construction, scene parsing
- GPU: Ray tracing, intersection tests
- Automatic selection based on scene complexity

## Troubleshooting

### "Failed to find suitable GPU adapter"
**Cause**: No compatible GPU found or drivers not installed.
**Solution**: 
- Update GPU drivers
- Check if your GPU supports Vulkan/Metal/DX12
- Use CPU rendering as fallback

### "Failed to initialize GPU"
**Cause**: wgpu couldn't initialize the graphics backend.
**Solution**:
- Try running with `WGPU_BACKEND=vulkan` (Linux)
- Try running with `WGPU_BACKEND=dx12` (Windows)
- Check GPU driver logs

### "GPU rendering failed: currently only supports spheres"
**Cause**: Scene contains triangles or planes.
**Solution**: Use CPU rendering for now, or convert geometry to spheres.

### Performance is slower on GPU
**Cause**: Small scenes have GPU initialization overhead.
**Solution**: 
- Use CPU rendering for small scenes
- Increase scene complexity to benefit from GPU parallelism

## Architecture Details

### GPU Pipeline

1. **Upload Phase** (once per scene):
   - Scene parameters → Uniform buffer
   - Camera data → Uniform buffer
   - Spheres → Storage buffer
   - Lights → Storage buffer

2. **Compute Phase** (GPU):
   - Dispatch compute shader (8×8 workgroups)
   - Each thread processes one pixel
   - Ray generation from camera
   - BVH-like traversal (sphere-specific)
   - Lighting calculations
   - Reflection recursion (up to max depth)

3. **Download Phase** (once per frame):
   - Read back pixel data from GPU
   - Convert to image format

### WGSL Shader

The compute shader (`src/raytracer/ray_tracing.wgsl`) implements:
- Ray-sphere intersection
- Closest hit detection
- Shadow ray testing
- Blinn-Phong lighting
- Reflection rays

### Memory Layout

All GPU buffers use explicit padding for proper alignment:
```rust
#[repr(C)]
#[derive(Pod, Zeroable)]
struct GPUSphere {
    center: [f32; 3],
    radius: f32,
    diffuse: [f32; 3],
    _padding1: f32,  // Align to 16 bytes
    specular: [f32; 3],
    shininess: f32,
}
```

## Future Enhancements

Planned improvements (see `GPU_ACCELERATION_INVESTIGATION.md` for details):

1. **Triangle Support**: Implement BVH traversal on GPU
2. **Full Feature Parity**: Support all shape types
3. **Optimization**: Better memory coalescing, workgroup sizing
4. **Progressive Rendering**: Show preview while rendering
5. **Ray Tracing Hardware**: Use RTX ray tracing cores when available
6. **Multi-GPU**: Distribute work across multiple GPUs

## Performance Benchmarks

See `GPU_ACCELERATION_INVESTIGATION.md` for detailed performance analysis and benchmarks.

## Related Files

- `src/raytracer/gpu_renderer.rs` - GPU renderer implementation
- `src/raytracer/ray_tracing.wgsl` - WGSL compute shader
- `GPU_ACCELERATION_INVESTIGATION.md` - Detailed investigation and design
- `examples/gpu_demo.rs` - Performance comparison example
- `gpu_test.scene` - Simple test scene for GPU rendering

## Contributing

To extend GPU support:

1. Review `GPU_ACCELERATION_INVESTIGATION.md` for architecture
2. Modify `ray_tracing.wgsl` for shader changes
3. Update `gpu_renderer.rs` for buffer/pipeline changes
4. Add tests comparing CPU vs GPU output
5. Update this README with new features

## References

- [wgpu Documentation](https://wgpu.rs/)
- [WGSL Specification](https://www.w3.org/TR/WGSL/)
- [Learn wgpu](https://sotrh.github.io/learn-wgpu/)
- Main project documentation: See root README
