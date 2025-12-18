# BVH (Bounding Volume Hierarchy) Implementation

## Overview

This ray tracer now uses a **Bounding Volume Hierarchy (BVH)** acceleration structure to significantly improve rendering performance. The BVH reduces the computational complexity of ray-object intersection tests from O(n) to O(log n) on average, where n is the number of objects in the scene.

## What is BVH?

A Bounding Volume Hierarchy is a tree structure that organizes scene objects based on their spatial positions. Each node in the tree contains:
- An axis-aligned bounding box (AABB) that encompasses all objects in its subtree
- References to child nodes (for internal nodes) or actual scene objects (for leaf nodes)

When a ray traverses the scene, the BVH allows us to quickly skip large groups of objects by testing against their bounding boxes first. If a ray doesn't intersect a node's bounding box, we can eliminate all objects within that node from consideration.

## Implementation Details

### Libraries Used

- **bvh v0.12.0**: Rust crate providing BVH construction and traversal
  - Uses Surface Area Heuristic (SAH) for optimal tree construction
  - Supports parallel construction via Rayon
  - Provides efficient ray-AABB intersection tests

- **nalgebra v0.34**: Math library used throughout the project
  - Unified math library for both the project and BVH crate
  - Provides Vector3<f32> for 3D vectors and Point3<f32> for points
  - Comprehensive linear algebra operations

### Key Components

#### 1. Shape Trait Implementations (`src/raytracer/config/shape.rs`)

Each shape type (Sphere, Triangle, Plane) implements two BVH-related traits:

- **`Bounded<f32, 3>`**: Provides an axis-aligned bounding box (AABB) for the shape
  - Spheres: Cube centered at sphere center with side length 2×radius
  - Triangles: Minimum box containing all three vertices
  - Planes: Very large box (limitation: infinite primitives don't benefit from BVH)

- **`BHShape<f32, 3>`**: Allows shapes to store their position in the BVH tree
  - Each shape has a `node_index` field for BVH bookkeeping

#### 2. RayTracer Integration (`src/raytracer/raytracer.rs`)

The `RayTracer` struct now contains a `Bvh<f32, 3>` field that is built during initialization:

```rust
pub struct RayTracer {
    config: Config,
    bvh: Bvh<f32, 3>,  // BVH acceleration structure
}
```

**Construction** (in `RayTracer::new()`):
- Uses `Bvh::build_par()` for parallel construction
- Built once at initialization time
- Objects are modified in-place to store their BVH indices

**Traversal** (in `find_color_recursive()`):
- Primary rays: BVH traversal identifies candidate objects for intersection
- Shadow rays: BVH traversal for fast occlusion testing
- Reflection rays: BVH traversal for recursive ray tracing

### Math Library

The project uses **nalgebra v0.34** as the unified math library for both the ray tracer and BVH:

- **Vector3<f32>**: 3D vectors for directions, colors, normals
- **Point3<f32>**: 3D points for positions
- All vector operations use nalgebra's API (dot products, cross products, normalization)
- No conversion overhead between different math libraries

## Performance Benefits

### Complexity Analysis

- **Without BVH**: O(n) intersection tests per ray
  - Every ray tests against every object
  - Performance degrades linearly with scene complexity

- **With BVH**: O(log₂ n) intersection tests per ray (average case)
  - BVH traversal eliminates large portions of the scene
  - Performance scales logarithmically with scene complexity

### Real-World Impact

For a scene with 1000 objects:
- **Without BVH**: ~1000 intersection tests per ray
- **With BVH**: ~10 intersection tests per ray (log₂(1000) ≈ 10)
- **Speedup**: ~100× reduction in intersection tests

The actual speedup depends on:
- Scene geometry distribution
- BVH construction quality (SAH provides near-optimal trees)
- Ray coherence (rays in similar directions benefit from cache locality)

## Testing and Validation

### Correctness Tests
All existing tests pass with BVH enabled, ensuring:
- Rendered images are identical to reference images
- No visual artifacts introduced
- Correct handling of shadows, reflections, and lighting

### Performance Benchmark
A dedicated benchmark test (`test_bvh_performance_benchmark`) measures rendering time and reports:
- Scene complexity (number of objects)
- Rendering duration
- Theoretical vs actual complexity

Run with: `cargo test test_bvh_performance_benchmark -- --nocapture`

## Limitations and Future Work

### Current Limitations
1. **Infinite Primitives**: Planes use very large AABBs and don't benefit from BVH
2. **Static Scenes**: BVH is rebuilt if scene changes (no dynamic updates)
3. **Memory Overhead**: BVH tree structure requires additional memory

### Potential Improvements
1. **BVH Optimization**: Use `bvh.optimize()` for scenes with minor updates
2. **Spatial Splits**: Consider implementing spatial splitting for large primitives
3. **GPU Traversal**: Flatten BVH for GPU ray tracing (using `FlatBvh`)
4. **SIMD**: Enable `simd` feature flag on nightly for explicit SIMD optimizations

## References

- [BVH Crate Documentation](https://docs.rs/bvh/)
- [Wikipedia: Bounding Volume Hierarchy](https://en.wikipedia.org/wiki/Bounding_volume_hierarchy)
- [Ray Tracing in One Weekend](https://raytracing.github.io/) - Peter Shirley
- [Surface Area Heuristic (SAH)](https://en.wikipedia.org/wiki/Bounding_volume_hierarchy#Optimization)

## Usage Example

The BVH is automatically built and used - no code changes needed:

```rust
// Create config from scene file
let mut parsed_config = ParsedConfigState::new();
let config = parsed_config.load_config_file("scene.test")?;

// BVH is built automatically in new()
let ray_tracer = RayTracer::new(config);

// Rendering uses BVH for all ray-object intersection queries
let image = ray_tracer.render()?;
```

## Building and Running

```bash
# Build with BVH support (enabled by default)
cargo build --release

# Run all tests including BVH validation
cargo test

# Run performance benchmark
cargo test test_bvh_performance_benchmark -- --nocapture --show-output

# Render a scene
cargo run --release
```
