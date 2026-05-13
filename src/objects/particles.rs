use std::sync::Arc;

use glam::Vec3;
use rand::Rng;

use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::objects::Sphere;
use crate::ray::Ray;

/// Many small spheres in a region (dust / sparkles). Each particle shares the same material Arc.
pub struct ParticleCloud {
    pub spheres: Vec<Sphere>,
}

impl ParticleCloud {
    pub fn random_box<R: Rng + ?Sized>(
        rng: &mut R,
        min: Vec3,
        max: Vec3,
        count: usize,
        radius: f32,
        material: Arc<Material>,
    ) -> Self {
        let mut spheres = Vec::with_capacity(count);
        for _ in 0..count {
            let c = Vec3::new(
                rng.gen_range(min.x..max.x),
                rng.gen_range(min.y..max.y),
                rng.gen_range(min.z..max.z),
            );
            spheres.push(Sphere::new(c, radius, material.clone()));
        }
        Self { spheres }
    }
}

impl Hittable for ParticleCloud {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut best: Option<HitRecord> = None;
        for s in &self.spheres {
            if let Some(h) = s.hit(ray, t_min, closest) {
                closest = h.t;
                best = Some(h);
            }
        }
        best
    }
}
