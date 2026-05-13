use std::sync::Arc;

use glam::Vec3;

use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;

/// Wavy heightfield `y = base_y + amp * sin(kx * x + ox) * cos(kz * z + oz)` approximated by ray marching.
pub struct FluidSurface {
    pub base_y: f32,
    pub amplitude: f32,
    pub kx: f32,
    pub kz: f32,
    pub phase_x: f32,
    pub phase_z: f32,
    pub material: Arc<Material>,
    pub step: f32,
    pub max_t: f32,
}

impl FluidSurface {
    pub fn height(&self, x: f32, z: f32) -> f32 {
        self.base_y
            + self.amplitude
                * (self.kx * x + self.phase_x).sin()
                * (self.kz * z + self.phase_z).cos()
    }

    pub fn normal(&self, x: f32, z: f32) -> Vec3 {
        let eps = 0.01;
        let hx = self.height(x + eps, z) - self.height(x - eps, z);
        let hz = self.height(x, z + eps) - self.height(x, z - eps);
        Vec3::new(-hx, 2.0 * eps, -hz).normalize_or_zero()
    }
}

impl Hittable for FluidSurface {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // March along ray until crossing surface from above (water seen from air).
        let mut t = t_min.max(0.001);
        let step = self.step.max(0.002);
        let p0 = ray.at(t);
        let mut prev_above = p0.y >= self.height(p0.x, p0.z);

        while t < t_max.min(self.max_t) {
            t += step;
            let p = ray.at(t);
            let h = self.height(p.x, p.z);
            let above = p.y >= h;
            if prev_above && !above {
                // Bisection refine
                let mut lo = t - step;
                let mut hi = t;
                for _ in 0..12 {
                    let mid = 0.5 * (lo + hi);
                    let pm = ray.at(mid);
                    if pm.y >= self.height(pm.x, pm.z) {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let tf = 0.5 * (lo + hi);
                if tf <= t_min || tf >= t_max {
                    return None;
                }
                let p = ray.at(tf);
                let n = self.normal(p.x, p.z);
                let mut rec = HitRecord {
                    t: tf,
                    p,
                    normal: Vec3::ZERO,
                    front_face: true,
                    uv: (p.x * 0.1 + 0.5, p.z * 0.1 + 0.5),
                    material: self.material.clone(),
                };
                rec.set_face_normal(ray, n);
                return Some(rec);
            }
            prev_above = above;
        }
        None
    }
}
