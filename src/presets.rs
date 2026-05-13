use std::sync::Arc;

use glam::Vec3;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::camera::Camera;
use crate::hit::HittableList;
use crate::material::Material;
use crate::objects::{Aabb, Cylinder, FluidSurface, ParticleCloud, Plane, Sphere};
use crate::scene::{PointLight, Scene};
use crate::texture::{CheckerTexture, NoiseTexture};

#[derive(Clone, Debug, Default)]
pub struct RenderSettings {
    pub textures: bool,
    pub reflection: bool,
    pub refraction: bool,
    pub particles: bool,
    pub fluid: bool,
}

/// Positions and scales for preset geometry. Defaults match the original auditor scenes.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectPlacement {
    /// `Preset::Sphere` — sphere center and radius.
    pub solo_sphere_center: Vec3,
    pub solo_sphere_radius: f32,
    /// `Preset::PlaneCube` — floor height (Y) and cube.
    pub pc_plane_y: f32,
    pub pc_cube_center: Vec3,
    pub pc_cube_half: f32,
    /// Multi-object presets — floor and three solids.
    pub ao_floor_y: f32,
    pub ao_sphere_center: Vec3,
    pub ao_sphere_radius: f32,
    pub ao_cube_center: Vec3,
    pub ao_cube_half: f32,
    pub ao_cylinder_center: Vec3,
    pub ao_cylinder_radius: f32,
    pub ao_cylinder_half_height: f32,
    /// Wavy fluid patch (when fluid bonus is on).
    pub ao_fluid_base_y: f32,
    pub ao_fluid_amplitude: f32,
    pub ao_fluid_kx: f32,
    pub ao_fluid_kz: f32,
}

impl ObjectPlacement {
    /// Layout used by `--render-auditor` and CLI defaults.
    pub fn auditor() -> Self {
        Self {
            solo_sphere_center: Vec3::new(1.0, 1.0, 1.0),
            solo_sphere_radius: 0.65,
            pc_plane_y: -0.75,
            pc_cube_center: Vec3::new(0.0, -0.2, 0.0),
            pc_cube_half: 0.55,
            ao_floor_y: -1.0,
            ao_sphere_center: Vec3::new(-1.1, 0.05, 0.4),
            ao_sphere_radius: 0.55,
            ao_cube_center: Vec3::new(0.85, 0.1, -0.35),
            ao_cube_half: 0.45,
            ao_cylinder_center: Vec3::new(0.0, 0.35, -1.15),
            ao_cylinder_radius: 0.42,
            ao_cylinder_half_height: 0.85,
            ao_fluid_base_y: -0.35,
            ao_fluid_amplitude: 0.06,
            ao_fluid_kx: 2.2,
            ao_fluid_kz: 2.6,
        }
    }

    pub fn for_preset(_preset: Preset) -> Self {
        Self::auditor()
    }
}

impl Default for ObjectPlacement {
    fn default() -> Self {
        Self::auditor()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Preset {
    Sphere,
    PlaneCube,
    AllObjects,
    AllObjectsAltCam,
    /// Quick preview: all shapes + bonuses, small default resolution via CLI.
    Showcase,
}

pub fn aspect_wh(width: u32, height: u32) -> f32 {
    width as f32 / height as f32
}

fn checker_tex(scale: f32) -> Arc<dyn crate::texture::Texture> {
    Arc::new(CheckerTexture {
        even: Vec3::new(0.15, 0.18, 0.22),
        odd: Vec3::new(0.85, 0.88, 0.92),
        scale,
    })
}

fn noise_tex() -> Arc<dyn crate::texture::Texture> {
    Arc::new(NoiseTexture::new(
        7,
        0.35,
        Vec3::new(0.25, 0.45, 0.85),
        Vec3::new(0.9, 0.95, 1.0),
    ))
}

/// Single sphere; bright lighting (auditor image 1).
pub fn scene_sphere(
    width: u32,
    height: u32,
    settings: &RenderSettings,
    placement: &ObjectPlacement,
) -> (Scene, RenderSettings) {
    let aspect = aspect_wh(width, height);
    let mut world = HittableList::new();

    let mut mat_sphere = Material::lambertian(Vec3::new(0.95, 0.35, 0.35));
    if settings.textures {
        mat_sphere.texture = Some(noise_tex());
    }
    if settings.reflection {
        mat_sphere.reflectivity = 0.35;
        mat_sphere.shininess = 64.0;
    }
    let mat_sphere = Arc::new(mat_sphere);

    let c = placement.solo_sphere_center;
    let r = placement.solo_sphere_radius.max(0.05);
    world.add(Arc::new(Sphere::new(c, r, mat_sphere)));

    // Keep a similar eye offset as the original preset when the sphere moves.
    let eye = c + Vec3::new(3.0, 1.8, 5.0);
    let camera = Camera::new(eye, c, Vec3::Y, 42.0, aspect);

    let scene = Scene {
        world,
        lights: vec![PointLight::new(
            Vec3::new(6.0, 8.0, 4.0),
            Vec3::ONE,
            2.4,
        )],
        camera,
        brightness: 1.15,
        ambient: 0.08,
        ambient_color: Vec3::new(0.35, 0.45, 0.65),
        background_top: Vec3::new(0.55, 0.72, 1.0),
        background_bottom: Vec3::new(0.95, 0.95, 1.0),
        max_trace_depth: 8,
    };
    (scene, settings.clone())
}

/// Ground plane + cube; **lower** brightness than sphere scene (auditor image 2).
pub fn scene_plane_cube(
    width: u32,
    height: u32,
    settings: &RenderSettings,
    placement: &ObjectPlacement,
) -> (Scene, RenderSettings) {
    let aspect = aspect_wh(width, height);
    let mut world = HittableList::new();

    let mut floor_mat = Material::lambertian(Vec3::splat(0.9));
    if settings.textures {
        floor_mat.texture = Some(checker_tex(0.35));
    }
    let floor_mat = Arc::new(floor_mat);

    world.add(Arc::new(Plane::horizontal(placement.pc_plane_y, floor_mat)));

    let mut cube_mat = Material::lambertian(Vec3::new(0.25, 0.55, 0.95));
    cube_mat.shininess = 48.0;
    if settings.reflection {
        cube_mat.reflectivity = 0.25;
    }
    let cube_mat = Arc::new(cube_mat);
    let half = placement.pc_cube_half.max(0.05);
    world.add(Arc::new(Aabb::cube(placement.pc_cube_center, half, cube_mat)));

    let camera = Camera::new(
        Vec3::new(-2.2, 1.4, 3.4),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::Y,
        50.0,
        aspect,
    );

    let scene = Scene {
        world,
        lights: vec![PointLight::new(
            Vec3::new(3.0, 5.5, 2.5),
            Vec3::ONE,
            0.85,
        )],
        camera,
        brightness: 0.55,
        ambient: 0.04,
        ambient_color: Vec3::new(0.25, 0.28, 0.35),
        background_top: Vec3::new(0.45, 0.55, 0.75),
        background_bottom: Vec3::new(0.9, 0.9, 0.92),
        max_trace_depth: 8,
    };
    (scene, settings.clone())
}

/// One of each primitive (auditor image 3).
pub fn scene_all_objects(
    _width: u32,
    _height: u32,
    settings: &RenderSettings,
    camera: Camera,
    placement: &ObjectPlacement,
) -> (Scene, RenderSettings) {
    let mut world = HittableList::new();

    let mut floor_mat = Material::lambertian(Vec3::splat(0.92));
    if settings.textures {
        floor_mat.texture = Some(checker_tex(0.28));
    }
    let floor_mat = Arc::new(floor_mat);
    world.add(Arc::new(Plane::horizontal(placement.ao_floor_y, floor_mat)));

    let mut sphere_mat = Material::lambertian(Vec3::new(0.95, 0.45, 0.35));
    sphere_mat.shininess = 64.0;
    if settings.reflection {
        sphere_mat.reflectivity = 0.45;
    }
    if settings.refraction {
        sphere_mat.ior = 1.45;
        sphere_mat.reflectivity = sphere_mat.reflectivity.max(0.08);
    }
    if settings.textures {
        sphere_mat.texture = Some(noise_tex());
    }
    let sr = placement.ao_sphere_radius.max(0.05);
    world.add(Arc::new(Sphere::new(
        placement.ao_sphere_center,
        sr,
        Arc::new(sphere_mat),
    )));

    let mut cube_mat = Material::lambertian(Vec3::new(0.35, 0.75, 0.45));
    if settings.reflection {
        cube_mat.reflectivity = 0.2;
    }
    let ch = placement.ao_cube_half.max(0.05);
    world.add(Arc::new(Aabb::cube(
        placement.ao_cube_center,
        ch,
        Arc::new(cube_mat),
    )));

    let cyl_mat = Arc::new(Material::metal(Vec3::new(0.82, 0.78, 0.55), 0.65));
    let cyl_r = placement.ao_cylinder_radius.max(0.05);
    let cyl_h = placement.ao_cylinder_half_height.max(0.05);
    world.add(Arc::new(Cylinder::new(
        placement.ao_cylinder_center,
        cyl_r,
        cyl_h,
        cyl_mat,
    )));

    if settings.fluid {
        let water = Arc::new(Material::glass(Vec3::new(0.75, 0.9, 1.0), 1.33));
        world.add(Arc::new(FluidSurface {
            base_y: placement.ao_fluid_base_y,
            amplitude: placement.ao_fluid_amplitude.max(0.001),
            kx: placement.ao_fluid_kx.max(0.01),
            kz: placement.ao_fluid_kz.max(0.01),
            phase_x: 0.4,
            phase_z: -0.2,
            material: water,
            step: 0.04,
            max_t: 24.0,
        }));
    }

    if settings.particles {
        let mut rng = StdRng::seed_from_u64(42);
        let dust = Arc::new(Material {
            albedo: Vec3::new(1.0, 0.95, 0.55),
            emission: Vec3::new(0.35, 0.3, 0.15),
            shininess: 8.0,
            ..Default::default()
        });
        let cloud = ParticleCloud::random_box(
            &mut rng,
            Vec3::new(-2.0, 0.2, -2.0),
            Vec3::new(2.0, 2.2, 2.0),
            900,
            0.018,
            dust,
        );
        world.add(Arc::new(cloud));
    }

    let scene = Scene {
        world,
        lights: vec![
            PointLight::new(Vec3::new(4.0, 7.0, 5.0), Vec3::ONE, 1.35),
            PointLight::new(Vec3::new(-4.0, 4.0, -2.0), Vec3::new(0.55, 0.65, 1.0), 0.35),
        ],
        camera,
        brightness: 1.0,
        ambient: 0.06,
        ambient_color: Vec3::new(0.3, 0.35, 0.45),
        background_top: Vec3::new(0.55, 0.72, 1.0),
        background_bottom: Vec3::new(0.92, 0.94, 1.0),
        max_trace_depth: 10,
    };
    (scene, settings.clone())
}

pub fn build_scene(
    preset: Preset,
    width: u32,
    height: u32,
    settings: &RenderSettings,
    placement: &ObjectPlacement,
) -> (Scene, RenderSettings) {
    let aspect = aspect_wh(width, height);
    match preset {
        Preset::Sphere => scene_sphere(width, height, settings, placement),
        Preset::PlaneCube => scene_plane_cube(width, height, settings, placement),
        Preset::AllObjects => {
            let cam = Camera::new(
                Vec3::new(2.8, 2.0, 5.0),
                Vec3::new(0.0, 0.15, 0.0),
                Vec3::Y,
                48.0,
                aspect,
            );
            scene_all_objects(width, height, settings, cam, placement)
        }
        Preset::AllObjectsAltCam => {
            let cam = Camera::orbit(
                Vec3::new(0.0, 0.15, 0.0),
                6.2,
                -52.0,
                2.4,
                48.0,
                aspect,
            );
            scene_all_objects(width, height, settings, cam, placement)
        }
        Preset::Showcase => {
            let s = RenderSettings {
                textures: true,
                reflection: true,
                refraction: true,
                particles: true,
                fluid: true,
            };
            let cam = Camera::new(
                Vec3::new(3.2, 2.2, 5.5),
                Vec3::new(0.0, 0.1, 0.0),
                Vec3::Y,
                50.0,
                aspect,
            );
            scene_all_objects(width, height, &s, cam, placement)
        }
    }
}

/// For GUI / tooling: multi-object presets use an orbit camera; sphere and plane-cube presets ignore orbit fields.
pub fn build_scene_interactive(
    preset: Preset,
    width: u32,
    height: u32,
    settings: &RenderSettings,
    look_at: Vec3,
    orbit_distance: f32,
    yaw_deg: f32,
    eye_height: f32,
    vfov_deg: f32,
    placement: &ObjectPlacement,
) -> (Scene, RenderSettings) {
    let aspect = aspect_wh(width, height);
    match preset {
        Preset::Sphere => scene_sphere(width, height, settings, placement),
        Preset::PlaneCube => scene_plane_cube(width, height, settings, placement),
        Preset::AllObjects | Preset::AllObjectsAltCam | Preset::Showcase => {
            let eff = if matches!(preset, Preset::Showcase) {
                RenderSettings {
                    textures: true,
                    reflection: true,
                    refraction: true,
                    particles: true,
                    fluid: true,
                }
            } else {
                settings.clone()
            };
            let cam = Camera::orbit(
                look_at,
                orbit_distance,
                yaw_deg,
                eye_height,
                vfov_deg,
                aspect,
            );
            scene_all_objects(width, height, &eff, cam, placement)
        }
    }
}
