use std::sync::Arc;

use glam::Vec3;

use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;

/// Infinite plane: `dot(n, x) = d` with unit normal `n`.
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
    pub material: Arc<Material>,
}

impl Plane {
    pub fn from_point_normal(point: Vec3, normal: Vec3, material: Arc<Material>) -> Self {
        let n = normal.normalize_or_zero();
        let d = point.dot(n);
        Self {
            normal: n,
            d,
            material,
        }
    }

    /// Horizontal plane through `y` with normal +Y.
    pub fn horizontal(y: f32, material: Arc<Material>) -> Self {
        Self::from_point_normal(Vec3::new(0.0, y, 0.0), Vec3::Y, material)
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let denom = ray.direction.dot(self.normal);
        if denom.abs() < 1e-8 {
            return None;
        }
        let t = (self.d - ray.origin.dot(self.normal)) / denom;
        if t <= t_min || t >= t_max {
            return None;
        }
        let p = ray.at(t);
        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3::ZERO,
            front_face: true,
            uv: (0.0, 0.0),
            material: self.material.clone(),
        };
        rec.set_face_normal(ray, self.normal);
        // Planar UV from world XZ
        rec.uv = (p.x * 0.25 + 0.5, p.z * 0.25 + 0.5);
        Some(rec)
    }
}
