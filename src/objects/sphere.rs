use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3;

use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<Material>) -> Self {
        Self {
            center,
            radius: radius.max(1e-6),
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let disc = half_b * half_b - a * c;
        if disc < 0.0 {
            return None;
        }
        let sqrtd = disc.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root <= t_min || root >= t_max {
            root = (-half_b + sqrtd) / a;
            if root <= t_min || root >= t_max {
                return None;
            }
        }

        let p = ray.at(root);
        let outward = (p - self.center) / self.radius;
        let mut rec = HitRecord {
            t: root,
            p,
            normal: Vec3::ZERO,
            front_face: true,
            uv: (0.0, 0.0),
            material: self.material.clone(),
        };
        rec.set_face_normal(ray, outward);

        let lp = (p - self.center) / self.radius;
        let phi = lp.z.atan2(lp.x);
        let theta = (-lp.y).clamp(-1.0, 1.0).acos();
        let u = phi / (2.0 * PI) + 0.5;
        let v = theta / PI;
        rec.uv = (u, v);

        Some(rec)
    }
}
