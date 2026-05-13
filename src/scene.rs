use glam::Vec3;

use crate::camera::Camera;
use crate::color::Color;
use crate::hit::HittableList;
use crate::ray::Ray;

#[derive(Clone, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    /// Linear multiplier applied to `color`.
    pub intensity: f32,
    /// Quadratic distance falloff denominator factor: atten = intensity / (1 + k * dist^2).
    pub falloff_k: f32,
}

impl PointLight {
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            falloff_k: 0.02,
        }
    }
}

pub struct Scene {
    pub world: HittableList,
    pub lights: Vec<PointLight>,
    pub camera: Camera,
    /// Global exposure multiplier for direct lighting and ambient.
    pub brightness: f32,
    /// Ambient term strength (linear RGB scale on albedo).
    pub ambient: f32,
    pub ambient_color: Vec3,
    pub background_top: Vec3,
    pub background_bottom: Vec3,
    pub max_trace_depth: u32,
}

impl Scene {
    pub fn background(&self, ray: &Ray) -> Color {
        let t = 0.5 * (ray.direction.y.clamp(-1.0, 1.0) + 1.0);
        let c = self.background_bottom.lerp(self.background_top, t);
        Color::new(c.x, c.y, c.z)
    }
}
