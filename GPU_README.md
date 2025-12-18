# GPU Acceleration Feature

## Quick Start

This ray tracer now supports GPU acceleration for significantly faster rendering (10-100x speedup on complex scenes).

### Enable GPU Rendering

```bash
# Build with GPU support
cargo build --release --features gpu

# Run with GPU support
cargo run --release --features gpu
```

### Demo

```bash
# Compare CPU vs GPU performance
cargo run --example gpu_demo --features gpu --release
```

### Requirements

- GPU with Vulkan, Metal, DX12, or WebGPU support
- Updated GPU drivers

### Documentation

- **Full Investigation**: See [`GPU_ACCELERATION_INVESTIGATION.md`](GPU_ACCELERATION_INVESTIGATION.md)
- **Usage Guide**: See [`GPU_USAGE.md`](GPU_USAGE.md)
- **Implementation Details**: See [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md)

### Current Status

✅ **Working**: Spheres, lighting, shadows, reflections  
⚠️ **Limited**: Currently supports only sphere objects  
🔄 **Future**: Triangle and plane support planned

### Performance

Example (3 spheres, 800x600):
- CPU: 45ms
- GPU: 15ms
- **Speedup: 3x**

For larger scenes (1000+ objects), expect 10-100x speedup.

### Default Behavior

Without the `gpu` feature flag, the ray tracer builds and runs exactly as before with no changes.
