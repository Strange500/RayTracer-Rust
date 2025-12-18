use nalgebra::Vector3;

#[derive(Clone)]
pub enum Light {
    Point { position: Vector3<f32>, color: Vector3<f32> },
    Directional { direction: Vector3<f32>, color: Vector3<f32> },
}

impl Light {
    pub fn color(&self) -> Vector3<f32> {
        match self {
            Light::Point { color, .. } | Light::Directional { color, .. } => *color,
        }
    }
}
