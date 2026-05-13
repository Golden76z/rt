use glam::Vec3;
use std::sync::Arc;

use crate::texture::Texture;

#[derive(Clone)]
pub struct Material {
    /// Base tint multiplied with texture (if any).
    pub albedo: Vec3,
    pub texture: Option<Arc<dyn Texture>>,
    /// 0 = diffuse, 1 = mirror-like reflection mixed into shading.
    pub reflectivity: f32,
    /// 0 = opaque; >0 enables refraction (glass/water). Typical glass ~1.5 outside/air.
    pub ior: f32,
    /// Emissive color (linear); adds directly to outgoing radiance.
    pub emission: Vec3,
    /// Specular shininess for Blinn-Phong highlight.
    pub shininess: f32,
    pub specular_color: Vec3,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::splat(0.8),
            texture: None,
            reflectivity: 0.0,
            ior: 1.0,
            emission: Vec3::ZERO,
            shininess: 32.0,
            specular_color: Vec3::ONE,
        }
    }
}

impl Material {
    pub fn lambertian(albedo: Vec3) -> Self {
        Self {
            albedo,
            ..Default::default()
        }
    }

    pub fn metal(albedo: Vec3, reflectivity: f32) -> Self {
        Self {
            albedo,
            reflectivity: reflectivity.clamp(0.0, 1.0),
            shininess: 128.0,
            ..Default::default()
        }
    }

    pub fn glass(albedo: Vec3, ior: f32) -> Self {
        Self {
            albedo,
            reflectivity: 0.05,
            ior: ior.max(1.0),
            shininess: 256.0,
            ..Default::default()
        }
    }

    pub fn light(emission: Vec3) -> Self {
        Self {
            albedo: Vec3::ZERO,
            emission,
            ..Default::default()
        }
    }

    #[inline]
    pub fn surface_albedo(&self, u: f32, v: f32, p: Vec3) -> Vec3 {
        let base = self.albedo;
        if let Some(tex) = &self.texture {
            base * tex.value(u, v, p)
        } else {
            base
        }
    }
}

#[inline]
pub fn reflect_vec(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

/// Returns refracted direction or None if total internal reflection.
pub fn refract_vec(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Option<Vec3> {
    let cos_theta = (-uv).dot(n).clamp(-1.0, 1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let len_sq = r_out_perp.length_squared();
    if len_sq >= 1.0 {
        return None;
    }
    let r_out_parallel = -(1.0 - len_sq).sqrt() * n;
    let dir = r_out_perp + r_out_parallel;
    if dir.length_squared() < 1e-20 {
        return None;
    }
    Some(dir.normalize())
}

pub fn schlick_reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
