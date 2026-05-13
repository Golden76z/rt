use glam::Vec3;
use std::sync::Arc;

use crate::material::Material;
use crate::ray::Ray;

#[derive(Clone)]
pub struct HitRecord {
    pub t: f32,
    pub p: Vec3,
    pub normal: Vec3,
    pub front_face: bool,
    pub uv: (f32, f32),
    pub material: Arc<Material>,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: Arc<dyn Hittable>) {
        self.objects.push(obj);
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut rec: Option<HitRecord> = None;
        for obj in &self.objects {
            if let Some(h) = obj.hit(ray, t_min, closest) {
                closest = h.t;
                rec = Some(h);
            }
        }
        rec
    }
}

impl Default for HittableList {
    fn default() -> Self {
        Self::new()
    }
}
