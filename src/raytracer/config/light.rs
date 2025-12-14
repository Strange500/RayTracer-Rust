use glam::Vec3;

/// Represents a point light in the scene
/// 
/// # Fields
/// * `position` - The 3D position of the light in world space
/// * `intensity` - The RGB color/intensity of the light (values 0.0-1.0)
pub struct Light {
    pub position: Vec3,
    pub intensity: Vec3,
}

impl Light {
    /// Creates a new Light with the specified position and intensity
    pub fn new(position: Vec3, intensity: Vec3) -> Self {
        Light {
            position,
            intensity,
        }
    }
}