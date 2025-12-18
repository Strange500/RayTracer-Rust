use nalgebra::Vector3;

pub struct Camera {
    pub(crate) position: Vector3<f32>,
    pub(crate) look_at: Vector3<f32>,
    pub(crate) up: Vector3<f32>,
    pub(crate) fov: f32,
}

impl Camera {
    pub fn direction(&self) -> Vector3<f32> {
        (self.look_at - self.position).normalize()
    }
}
