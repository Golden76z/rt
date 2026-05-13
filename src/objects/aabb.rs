use std::sync::Arc;

use glam::Vec3;

use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;

/// Axis-aligned box (cube when edges equal), defined by min/max corners.
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Arc<Material>,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3, material: Arc<Material>) -> Self {
        Self { min, max, material }
    }

    /// Centered cube with half-extent `half`.
    pub fn cube(center: Vec3, half: f32, material: Arc<Material>) -> Self {
        let h = Vec3::splat(half);
        Self::new(center - h, center + h, material)
    }
}

impl Hittable for Aabb {
    fn hit(&self, ray: &Ray, ray_t_min: f32, ray_t_max: f32) -> Option<HitRecord> {
        let mut t0 = ray_t_min;
        let mut t1 = ray_t_max;

        for a in 0..3 {
            let inv_d = 1.0 / ray.direction[a];
            let t_near = (self.min[a] - ray.origin[a]) * inv_d;
            let t_far = (self.max[a] - ray.origin[a]) * inv_d;
            let (t_near, t_far) = if t_near < t_far {
                (t_near, t_far)
            } else {
                (t_far, t_near)
            };
            t0 = t0.max(t_near);
            t1 = t1.min(t_far);
            if t0 > t1 {
                return None;
            }
        }

        let t = t0;
        if t <= ray_t_min || t >= ray_t_max {
            return None;
        }

        let p = ray.at(t);
        let eps = 1e-4 * (self.max - self.min).max_element().max(1.0);

        let outward = if (p.x - self.min.x).abs() < eps {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (p.x - self.max.x).abs() < eps {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (p.y - self.min.y).abs() < eps {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (p.y - self.max.y).abs() < eps {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (p.z - self.min.z).abs() < eps {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };

        let mut rec = HitRecord {
            t,
            p,
            normal: Vec3::ZERO,
            front_face: true,
            uv: (0.0, 0.0),
            material: self.material.clone(),
        };
        rec.set_face_normal(ray, outward);

        // Face UVs in [0,1] per face
        if outward.x.abs() > 0.5 {
            rec.uv = ((p.z - self.min.z) / (self.max.z - self.min.z).max(1e-6), (p.y - self.min.y) / (self.max.y - self.min.y).max(1e-6));
        } else if outward.y.abs() > 0.5 {
            rec.uv = ((p.x - self.min.x) / (self.max.x - self.min.x).max(1e-6), (p.z - self.min.z) / (self.max.z - self.min.z).max(1e-6));
        } else {
            rec.uv = ((p.x - self.min.x) / (self.max.x - self.min.x).max(1e-6), (p.y - self.min.y) / (self.max.y - self.min.y).max(1e-6));
        }

        Some(rec)
    }
}
