use glam::Vec3;
pub enum Light {
    Point { position: Vec3, color: Vec3 },
    Directional { direction: Vec3, color: Vec3 },
}

impl Light {
    pub fn color(&self) -> Vec3 {
        match self {
            Light::Point { color, .. } | Light::Directional { color, .. } => *color,
        }
    }
}
