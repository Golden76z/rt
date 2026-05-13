use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3;

use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;

/// Right circular cylinder aligned with **Y**, from `y_min` to `y_max` in world space, radius at xz.
pub struct Cylinder {
    pub center_xz: Vec3,
    pub y_min: f32,
    pub y_max: f32,
    pub radius: f32,
    pub material: Arc<Material>,
}

impl Cylinder {
    pub fn new(center: Vec3, radius: f32, y_half_height: f32, material: Arc<Material>) -> Self {
        Self {
            center_xz: Vec3::new(center.x, 0.0, center.z),
            y_min: center.y - y_half_height,
            y_max: center.y + y_half_height,
            radius: radius.max(1e-6),
            material,
        }
    }
}

impl Hittable for Cylinder {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let ox = ray.origin.x - self.center_xz.x;
        let oz = ray.origin.z - self.center_xz.z;
        let dx = ray.direction.x;
        let dz = ray.direction.z;

        let a = dx * dx + dz * dz;
        let half_b = ox * dx + oz * dz;
        let c = ox * ox + oz * oz - self.radius * self.radius;

        let mut best: Option<HitRecord> = None;
        let mut closest = t_max;

        if a.abs() > 1e-12 {
            let disc = half_b * half_b - a * c;
            if disc >= 0.0 {
                let sqrtd = disc.sqrt();
                for root in [(-half_b - sqrtd) / a, (-half_b + sqrtd) / a] {
                    if root <= t_min || root >= closest {
                        continue;
                    }
                    let p = ray.at(root);
                    if p.y < self.y_min || p.y > self.y_max {
                        continue;
                    }
                    let outward = Vec3::new(p.x - self.center_xz.x, 0.0, p.z - self.center_xz.z)
                        / self.radius;
                    let mut rec = HitRecord {
                        t: root,
                        p,
                        normal: Vec3::ZERO,
                        front_face: true,
                        uv: (0.0, 0.0),
                        material: self.material.clone(),
                    };
                    rec.set_face_normal(ray, outward);
                    let theta = outward.z.atan2(outward.x);
                    let u = theta / (2.0 * PI) + 0.5;
                    let v = (p.y - self.y_min) / (self.y_max - self.y_min).max(1e-6);
                    rec.uv = (u, v);
                    closest = root;
                    best = Some(rec);
                }
            }
        }

        // Caps (disks)
        if ray.direction.y.abs() > 1e-8 {
            for y_cap in [self.y_min, self.y_max] {
                let t = (y_cap - ray.origin.y) / ray.direction.y;
                if t <= t_min || t >= closest {
                    continue;
                }
                let p = ray.at(t);
                let dx = p.x - self.center_xz.x;
                let dz = p.z - self.center_xz.z;
                if dx * dx + dz * dz > self.radius * self.radius + 1e-5 {
                    continue;
                }
                let outward = if y_cap > (self.y_min + self.y_max) * 0.5 {
                    Vec3::Y
                } else {
                    -Vec3::Y
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
                let u = (dx / self.radius) * 0.5 + 0.5;
                let v = (dz / self.radius) * 0.5 + 0.5;
                rec.uv = (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
                closest = t;
                best = Some(rec);
            }
        }

        best
    }
}
