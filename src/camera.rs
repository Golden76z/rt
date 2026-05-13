use glam::Vec3;

use crate::ray::Ray;

/// Right-handed Y-up camera. `look_from` is the eye, `look_at` is the target point.
#[derive(Clone, Debug)]
pub struct Camera {
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub vup: Vec3,
    pub vfov_deg: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, vup: Vec3, vfov_deg: f32, aspect: f32) -> Self {
        Self {
            look_from,
            look_at,
            vup,
            vfov_deg,
            aspect,
        }
    }

    /// Ray through normalized image plane coords (u,v) in [0,1]^2, origin top-left of image.
    pub fn ray(&self, u: f32, v: f32) -> Ray {
        let w = (self.look_from - self.look_at).normalize_or_zero();
        let u_axis = self.vup.cross(w).normalize_or_zero();
        let v_axis = w.cross(u_axis);

        let theta = self.vfov_deg.to_radians();
        let h = (theta * 0.5).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = self.aspect * viewport_height;

        let lower_left_corner = self.look_from
            - u_axis * (viewport_width * 0.5)
            - v_axis * (viewport_height * 0.5)
            - w;

        let hor = u_axis * viewport_width;
        let ver = v_axis * viewport_height;
        let rd = (lower_left_corner + hor * u + ver * (1.0 - v) - self.look_from).normalize_or_zero();
        Ray::new(self.look_from, rd)
    }

    /// Orbit `look_at` on the XZ plane by `yaw_deg` (degrees), keeping distance and height style from default.
    pub fn orbit(look_at: Vec3, distance: f32, yaw_deg: f32, height: f32, vfov_deg: f32, aspect: f32) -> Self {
        let yaw = yaw_deg.to_radians();
        let look_from = look_at
            + Vec3::new(distance * yaw.sin(), height, distance * yaw.cos());
        Self::new(look_from, look_at, Vec3::Y, vfov_deg, aspect)
    }
}
