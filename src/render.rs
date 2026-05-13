use glam::Vec3;
use rayon::prelude::*;

use crate::color::Color;
use crate::material::{reflect_vec, refract_vec, schlick_reflectance};
use crate::presets::RenderSettings;
use crate::ray::Ray;
use crate::scene::Scene;

const SHADOW_BIAS: f32 = 1e-3;

fn in_shadow(scene: &Scene, p: Vec3, n: Vec3, light_pos: Vec3) -> bool {
    let to = light_pos - p;
    let dist = to.length();
    if dist < 1e-4 {
        return false;
    }
    let dir = to / dist;
    let origin = p + n * SHADOW_BIAS + dir * SHADOW_BIAS;
    let ray = Ray::new(origin, dir);
    scene
        .world
        .hit(&ray, 0.001, dist - 2.0 * SHADOW_BIAS)
        .is_some()
}

pub fn shade(scene: &Scene, ray: &Ray, depth: u32, settings: &RenderSettings) -> Color {
    if depth == 0 {
        return Color::BLACK;
    }

    let Some(hit) = scene.world.hit(ray, 0.001, f32::INFINITY) else {
        return scene.background(ray);
    };

    let n = hit.normal;
    let v = (-ray.direction).normalize_or_zero();
    let mat = hit.material.as_ref();

    let kd = if settings.textures {
        mat.surface_albedo(hit.uv.0, hit.uv.1, hit.p)
    } else {
        mat.albedo
    };

    let mut out = Color::from(mat.emission);

    out += Color::from(kd * scene.ambient_color * scene.ambient * scene.brightness);

    for light in &scene.lights {
        if in_shadow(scene, hit.p, n, light.position) {
            continue;
        }
        let to = light.position - hit.p;
        let dist_sq = to.length_squared().max(1e-4);
        let dist = dist_sq.sqrt();
        let ldir = to / dist;
        let ndl = n.dot(ldir).max(0.0);
        let att = light.intensity / (1.0 + light.falloff_k * dist_sq);
        let scale = att * scene.brightness * ndl;
        out += Color::from(light.color * scale) * kd;

        let h = (ldir + v).normalize_or_zero();
        if h.length_squared() > 1e-12 {
            let ndh = n.dot(h).max(0.0).powf(mat.shininess.max(1.0));
            out += Color::from(mat.specular_color * light.color * (ndh * scale * scene.brightness));
        }
    }

    let reflectivity = if settings.reflection {
        mat.reflectivity
    } else {
        0.0
    };

    if reflectivity > 1e-4 {
        let refl_dir = reflect_vec(ray.direction, n);
        if refl_dir.length_squared() > 1e-12 {
            let refl_ray = Ray::new(hit.p + n * SHADOW_BIAS, refl_dir);
            let reflected = shade(scene, &refl_ray, depth - 1, settings);
            out += reflected * reflectivity;
        }
    }

    let ior = if settings.refraction {
        mat.ior
    } else {
        1.0
    };

    if ior > 1.0001 {
        let ratio = if hit.front_face { 1.0 / ior } else { ior };
        let cos = (-ray.direction.dot(n)).clamp(-1.0, 1.0);
        let r0 = schlick_reflectance(cos, if hit.front_face { ior } else { 1.0 / ior });
        let refr_prob = (1.0 - r0).clamp(0.0, 1.0);

        if let Some(refr_dir) = refract_vec(ray.direction, n, ratio) {
            if refr_dir.length_squared() > 1e-12 {
                let refr_ray = Ray::new(hit.p - n * SHADOW_BIAS, refr_dir);
                let transmitted = shade(scene, &refr_ray, depth - 1, settings);
                let tint = kd.lerp(Vec3::ONE, 0.35);
                out += Color::from(transmitted.0 * tint) * (refr_prob * 0.95);
            }
        }
    }

    out
}

pub fn render_image(
    scene: &Scene,
    width: u32,
    height: u32,
    settings: &RenderSettings,
) -> Vec<Color> {
    let mut buf = vec![Color::BLACK; (width * height) as usize];
    buf.par_chunks_mut(width as usize)
        .enumerate()
        .for_each(|(row, line)| {
            let j = row as u32;
            for i in 0..width {
                let u = (i as f32 + 0.5) / width as f32;
                let v = (j as f32 + 0.5) / height as f32;
                let ray = scene.camera.ray(u, v);
                let c = shade(scene, &ray, scene.max_trace_depth, settings).linear_to_gamma();
                line[i as usize] = c;
            }
        });
    buf
}
