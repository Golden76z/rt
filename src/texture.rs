use glam::Vec3;
use noise::{NoiseFn, Seedable};
use std::sync::Arc;

/// UV in [0,1]^2; `p` is hit point in world space for procedural textures.
pub trait Texture: Send + Sync {
    fn value(&self, u: f32, v: f32, p: Vec3) -> Vec3;
}

pub struct SolidTexture {
    pub color: Vec3,
}

impl Texture for SolidTexture {
    fn value(&self, _u: f32, _v: f32, _p: Vec3) -> Vec3 {
        self.color
    }
}

pub struct CheckerTexture {
    pub even: Vec3,
    pub odd: Vec3,
    pub scale: f32,
}

impl Texture for CheckerTexture {
    fn value(&self, _u: f32, _v: f32, p: Vec3) -> Vec3 {
        let s = self.scale.max(1e-4);
        let x = (p.x * s).floor() as i32;
        let y = (p.y * s).floor() as i32;
        let z = (p.z * s).floor() as i32;
        if (x + y + z) & 1 == 0 {
            self.even
        } else {
            self.odd
        }
    }
}

/// Loads an image from disk; repeats UV.
pub struct ImageTexture {
    pub data: image::RgbImage,
}

impl ImageTexture {
    pub fn load(path: &str) -> std::io::Result<Arc<Self>> {
        let img = image::open(path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            .to_rgb8();
        Ok(Arc::new(Self { data: img }))
    }

    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let u = u.fract().abs();
        let v = 1.0 - v.fract().abs();
        let w = self.data.width() as f32;
        let h = self.data.height() as f32;
        let i = ((u * w).floor() as u32).min(self.data.width().saturating_sub(1));
        let j = ((v * h).floor() as u32).min(self.data.height().saturating_sub(1));
        let px = self.data.get_pixel(i, j);
        Vec3::new(
            px[0] as f32 / 255.0,
            px[1] as f32 / 255.0,
            px[2] as f32 / 255.0,
        )
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f32, v: f32, _p: Vec3) -> Vec3 {
        self.sample(u, v)
    }
}

/// Procedural noise tint using Perlin noise.
pub struct NoiseTexture {
    noise: noise::Perlin,
    pub scale: f64,
    pub tint_a: Vec3,
    pub tint_b: Vec3,
}

impl NoiseTexture {
    pub fn new(seed: u32, scale: f64, tint_a: Vec3, tint_b: Vec3) -> Self {
        Self {
            noise: noise::Perlin::default().set_seed(seed),
            scale,
            tint_a,
            tint_b,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f32, _v: f32, p: Vec3) -> Vec3 {
        let n = self.noise.get([
            (p.x as f64) * self.scale,
            (p.y as f64) * self.scale,
            (p.z as f64) * self.scale,
        ]);
        let t = 0.5 * (n + 1.0);
        self.tint_a.lerp(self.tint_b, t as f32)
    }
}
