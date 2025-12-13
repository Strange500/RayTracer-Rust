use glam::Vec3;

pub struct Camera {
    pub(crate) position: Vec3,
    pub(crate) look_at: Vec3,
    pub(crate) up: Vec3,
    pub(crate) fov: f32,
}