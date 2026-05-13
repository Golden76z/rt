# rt — Ray tracer documentation

This document explains what the ray tracer does, how to run it, and how to control scenes, lights, camera, and optional bonus features.

## What is implemented

- **Primitives**: sphere, axis-aligned cube (`Aabb`), infinite plane, finite **Y-up** cylinder (body + caps).
- **Camera**: position + look-at + vertical FOV; CLI presets and an **orbit** helper for turntable views.
- **Lighting**: point lights with distance falloff, **shadow rays**, ambient fill, Blinn–Phong highlights.
- **Materials**: textured or solid albedo, adjustable reflectivity, index of refraction (simple Fresnel / refraction), emissive particles.
- **Output**: ASCII **P3** `.ppm` (portable pixmap), any width/height.
- **Performance**: image rows are traced in parallel with **rayon**.
- **Bonuses (toggle)**:
  - **Textures**: checker floor, procedural noise tint, optional image maps (API in code).
  - **Reflection / refraction**: recursive shading up to `Scene::max_trace_depth`.
  - **Particles**: many small emissive spheres in a box (multi-object presets).
  - **Fluid**: wavy **heightfield** surface with ray marching (water-like refraction material).

## Build and run

```bash
cargo build --release
```

Release mode is strongly recommended for 800×600 and bonus features.

### Quick test (small resolution, stdout PPM)

```bash
cargo run --release -- --preset sphere --width 320 --height 240 > preview.ppm
```

Open `preview.ppm` in an image viewer that supports PPM (GIMP, IrfanView, VS Code viewers, etc.).

### Write to a file instead of stdout

```bash
cargo run --release -- --preset all-objects --width 800 --height 600 -o scene.ppm ^
  --textures --reflection --refraction --particles --fluid
```

(On Unix shells, replace `^` with `\`.)

### Generate the **four auditor images** (800×600, with bonuses where appropriate)

```bash
cargo run --release -- --render-auditor
```

Files appear under `images/`:

1. `01_sphere.ppm` — single sphere at **(1,1,1)** with brighter lighting.
2. `02_plane_cube_dim.ppm` — floor + cube with **lower** global brightness than (1).
3. `03_all_primitives.ppm` — sphere, cube, cylinder, plane **+** fluid + particles + reflections/refraction/textures.
4. `04_all_primitives_alt_camera.ppm` — same scene as (3), **different camera**.

### Global brightness

Each preset sets `Scene::brightness`, `ambient`, and light intensities. You can multiply the preset brightness from the CLI:

```bash
cargo run --release -- --preset plane-cube --brightness-mul 1.4 -o brighter.ppm
```

### Interactive GUI (optional)

```bash
cargo run --release --features gui -- --gui
```

Sliders cover export resolution, **preview resolution**, bonus toggles, orbit camera, brightness multiplier, and output path. The window is split: **controls on the left**, **rendered image on the right** (updated after every successful trace). Turn on **Auto-update preview** to retrace at the preview size after you stop changing parameters for ~420 ms (debounced so dragging a slider does not start hundreds of renders). Full export still uses **Render export → PPM** at the export width/height. Choose **Fixed preset camera** to match the CLI framing for a given `--preset`.

## Ray tracing model (short)

For each pixel the camera casts a **primary ray**. The closest hit along the ray yields a surface point and normal. **Direct lighting** sums contributions from each point light if an unoccluded **shadow ray** reaches the light. A small **ambient** term prevents shadows from clipping to pure black. **Reflection** and **refraction** spawn secondary rays recursively (Whitted-style) until the recursion limit is reached.

## Code examples (library usage)

Below examples assume `use rt::*;` and `use glam::Vec3;` plus `std::sync::Arc`.

### Sphere

```rust
let mat = Arc::new(Material::lambertian(Vec3::new(0.9, 0.2, 0.2)));
let sphere = Arc::new(objects::Sphere::new(Vec3::new(1.0, 1.0, 1.0), 0.6, mat));
world.add(sphere);
```

### Cube (axis-aligned box)

```rust
let mat = Arc::new(Material::lambertian(Vec3::new(0.2, 0.8, 0.3)));
let cube = Arc::new(objects::Aabb::cube(Vec3::new(0.0, 0.5, 0.0), 0.4, mat));
world.add(cube);
```

### Flat plane

```rust
let mat = Arc::new(Material::lambertian(Vec3::splat(0.85)));
let ground = Arc::new(objects::Plane::horizontal(-1.0, mat)); // y = -1, normal +Y
world.add(ground);
```

### Cylinder (Y axis, capped)

```rust
let mat = Arc::new(Material::metal(Vec3::new(0.9, 0.85, 0.4), 0.4));
let cyl = Arc::new(objects::Cylinder::new(
    Vec3::new(0.0, 0.4, -1.0), // center
    0.35,                      // radius
    0.7,                       // **half** height along Y
    mat,
));
world.add(cyl);
```

### Brightness (scene fields)

```rust
let scene = Scene {
    brightness: 1.0,                 // scales direct lights + user-facing exposure
    ambient: 0.06,                 // scales ambient term on materials
    ambient_color: Vec3::new(0.3, 0.35, 0.45),
    lights: vec![PointLight::new(Vec3::new(4.0, 8.0, 3.0), Vec3::ONE, 1.2)],
    // world + camera + backgrounds + max_trace_depth ...
    ../* see `src/presets.rs` */
};
```

Lower `brightness`, `ambient`, and/or light `intensity` for a dimmer image.

### Camera position and angle

```rust
let camera = Camera::new(
    Vec3::new(3.0, 2.0, 5.0), // eye
    Vec3::new(0.0, 0.0, 0.0), // look-at
    Vec3::Y,                  // up
    45.0,                     // vertical FOV (degrees)
    width as f32 / height as f32,
);
```

Turntable orbit around a point:

```rust
let camera = Camera::orbit(
    Vec3::new(0.0, 0.2, 0.0), // look-at
    6.0,                      // distance on XZ circle
    -40.0,                    // yaw (degrees)
    2.0,                      // eye height
    48.0,                     // vfov
    aspect,
);
```

## Moving and scaling objects (code + GUI)

Geometry is driven by **`ObjectPlacement`** in `src/presets.rs`. Defaults match the auditor layout (`ObjectPlacement::auditor()` / `Default`). Pass a reference into `build_scene` or `build_scene_interactive`:

```rust
use glam::Vec3;
use rt::presets::{build_scene, ObjectPlacement, Preset, RenderSettings};

let mut placement = ObjectPlacement::auditor();
placement.ao_sphere_center += Vec3::new(0.2, 0.0, 0.0);
let settings = RenderSettings::default();
let (scene, eff) = build_scene(Preset::AllObjects, 800, 600, &settings, &placement);
```

With **`--features gui`**, use the **“Objects — move & scale”** panel for sliders (which controls appear depends on the selected preset). Changing the **preset** resets placement to defaults.

## Bonus flags (CLI)

| Flag | Effect |
|------|--------|
| `--textures` | Sample procedural / assigned textures where materials provide them. |
| `--reflection` | Enable recursive reflections where `Material::reflectivity` > 0. |
| `--refraction` | Enable Snell refraction where `Material::ior` > 1. |
| `--particles` | Add emissive particle dust in multi-object presets. |
| `--fluid` | Add a ray-marched wavy water surface. |

## Project layout (important files)

- `src/render.rs` — shading, shadows, recursion.
- `src/presets.rs` — auditor scenes, `ObjectPlacement`, `build_scene` / `build_scene_interactive`.
- `src/objects/*` — geometry intersection.
- `src/material.rs`, `src/texture.rs` — surface models.
- `src/main.rs` — CLI.
- `src/gui.rs` — optional egui front-end (`--features gui`).

## Tips for auditors

- Use **`--release`**.
- For interactive iteration use small `--width/--height`, then render 800×600 for submission.
- The reference command for the four required images is `--render-auditor`.
