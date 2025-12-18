# GPU Acceleration Usage Guide

## Overview

This ray tracer now supports GPU-accelerated rendering using WebGPU (via `wgpu`). This can provide significant performance improvements for complex scenes with many objects and high resolutions.

## Requirements

- A GPU with Vulkan, Metal, DirectX 12, or WebGPU support
- GPU drivers installed and up-to-date
- Rust toolchain (for building)

## Usage

### CPU Rendering (Default)

```bash
# Render with CPU (default behavior)
cargo run --release [scene_file]

# Example:
cargo run --release simple_test.scene
```

### GPU Rendering

```bash
# Enable GPU rendering with --gpu flag
cargo run --release [scene_file] --gpu

# Example:
cargo run --release simple_test.scene --gpu
```

### Command Line Arguments

- `[scene_file]`: Path to the scene file (optional, defaults to `final_avec_bonus.scene`)
- `--gpu`: Enable GPU acceleration (optional, defaults to CPU rendering)

## Performance Expectations

### When to Use GPU:
- Complex scenes with 100+ objects
- High resolutions (1920x1080 or higher)
- Scenes with many triangles (e.g., mesh models)
- Multiple bounces (high maxdepth)

### When to Use CPU:
- Simple scenes (< 50 objects)
- Low resolutions (< 640x480)
- Development/debugging (easier to troubleshoot)
- Systems without GPU support

### Benchmark Results (Expected)

| Scene Complexity | CPU Time | GPU Time | Speedup |
|-----------------|----------|----------|---------|
| Simple (10 objects) | 50ms | 100ms | 0.5x (overhead) |
| Medium (100 objects) | 500ms | 100ms | 5x |
| Complex (1000 objects) | 5s | 250ms | 20x |
| Very Complex (10k+ triangles) | 60s | 3s | 20x |

*Note: Actual performance depends on hardware, scene complexity, and resolution.*

## Automatic Fallback

If GPU initialization fails (e.g., no GPU available, driver issues), the renderer automatically falls back to CPU rendering:

```
Starting GPU rendering...
Error during GPU rendering: Failed to find a suitable GPU adapter
Falling back to CPU rendering...
Starting CPU rendering...
```

## Implementation Details

### Architecture

```
RayTracer
├── render() - CPU rendering (Rayon parallel)
└── render_gpu() - GPU rendering (wgpu compute shader)
```

### GPU Backend Components

1. **gpu_backend.rs**: Main GPU backend implementation
   - Device initialization
   - Buffer management
   - Scene data upload
   - Compute shader execution

2. **shader.wgsl**: WebGPU Shading Language compute shader
   - Ray generation
   - Intersection tests (sphere, plane, triangle)
   - Lighting calculations (diffuse, specular)
   - Shadow rays
   - Iterative reflection (no recursion in shaders)

### Technical Details

- **Workgroup size**: 8x8 threads
- **Buffer types**: Uniform (camera, scene data), Storage (geometry, lights, output)
- **Reflection**: Implemented iteratively (max depth from scene config)
- **Precision**: 32-bit floating point (same as CPU)

## Troubleshooting

### GPU Not Found

```
Error: Failed to find a suitable GPU adapter
```

**Solutions:**
1. Update GPU drivers
2. Ensure Vulkan/Metal/DX12 support
3. Check GPU is not disabled in BIOS/OS
4. Use CPU rendering as fallback (automatic)

### Shader Compilation Errors

If you modify `shader.wgsl` and encounter errors, check:
1. WGSL syntax (different from GLSL/HLSL)
2. Buffer bindings match Rust structs
3. Struct alignment (use padding fields)

### Performance Issues

If GPU is slower than CPU:
1. Scene might be too simple (GPU overhead)
2. Check resolution (GPU shines at high res)
3. Ensure release build: `cargo build --release`
4. Profile with `--features profiling` (if added)

## Development

### Adding New Features

When extending the ray tracer:

1. **CPU changes**: Modify `raytracer.rs`
2. **GPU changes**: 
   - Update `gpu_backend.rs` (buffer structures)
   - Update `shader.wgsl` (shader code)
   - Keep CPU and GPU implementations in sync

### Testing

```bash
# Run all tests (uses CPU renderer)
cargo test

# Test specific scene
cargo run --release test.scene
cargo run --release test.scene --gpu

# Compare outputs
diff cpu_output.png gpu_output.png
```

### Debugging GPU Code

GPU shader debugging is challenging. Tips:

1. Use simple scenes for testing
2. Compare GPU vs CPU output pixel-by-pixel
3. Add debug colors in shader (e.g., visualize normals)
4. Use GPU debugging tools:
   - RenderDoc (capture GPU frames)
   - Metal Frame Debugger (macOS)
   - PIX (Windows)

## Future Improvements

Potential enhancements:

1. **Adaptive sampling**: Spend more time on complex pixels
2. **BVH acceleration**: Spatial data structure for faster intersection
3. **Denoising**: AI-based noise reduction
4. **Progressive rendering**: Show incremental results
5. **Multi-GPU**: Distribute work across multiple GPUs
6. **Async rendering**: Non-blocking render in background

## References

- [wgpu Documentation](https://wgpu.rs/)
- [WGSL Specification](https://www.w3.org/TR/WGSL/)
- [GPU_ACCELERATION_RESEARCH.md](GPU_ACCELERATION_RESEARCH.md) - Detailed research findings

## License

Same as the main project.
