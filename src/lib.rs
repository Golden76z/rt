//! Ray tracing library: shapes, materials, lights, rendering to PPM.

pub mod camera;
pub mod color;
pub mod hit;
pub mod material;
pub mod objects;
pub mod ppm;
pub mod presets;
pub mod ray;
pub mod render;
pub mod scene;
pub mod texture;

#[cfg(feature = "gui")]
pub mod gui;

pub use camera::Camera;
pub use color::Color;
pub use hit::{HitRecord, Hittable, HittableList};
pub use material::Material;
pub use presets::{ObjectPlacement, Preset, RenderSettings};
pub use ray::Ray;
pub use render::render_image;
pub use scene::{PointLight, Scene};
