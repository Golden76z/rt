use glam::Vec3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Color(pub Vec3);

impl Color {
    pub const BLACK: Color = Color(Vec3::ZERO);
    pub const WHITE: Color = Color(Vec3::ONE);

    #[inline]
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self(Vec3::new(r, g, b))
    }

    #[inline]
    pub fn linear_to_gamma(self) -> Self {
        let c = self.0.max(Vec3::ZERO);
        Self(Vec3::new(c.x.sqrt(), c.y.sqrt(), c.z.sqrt()))
    }

    pub fn to_u8_rgb(self) -> [u8; 3] {
        let c = self.0.clamp(Vec3::ZERO, Vec3::splat(1.0));
        [
            (c.x * 255.999) as u8,
            (c.y * 255.999) as u8,
            (c.z * 255.999) as u8,
        ]
    }
}

impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl std::ops::Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        Color(rhs.0 * self)
    }
}

impl std::ops::Mul<Vec3> for Color {
    type Output = Self;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl std::ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}
