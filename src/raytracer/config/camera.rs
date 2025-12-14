use glam::Vec3;

pub struct Camera {
    pub(crate) position: Vec3,
    pub(crate) look_at: Vec3,
    pub(crate) up: Vec3,
    pub(crate) fov: f32,
}

impl Camera {
    pub fn direction(&self) -> Vec3 {
        (self.look_at - self.position).normalize()
    }
}
