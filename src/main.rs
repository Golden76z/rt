use std::fs;
use std::io::{self, stdout};
use std::path::PathBuf;

use clap::Parser;
use rt::ppm::{write_ppm_file, write_ppm_p3};
use rt::presets::{build_scene, ObjectPlacement, Preset, RenderSettings};
use rt::render::render_image;

#[derive(Parser, Debug)]
#[command(name = "rt", version, about = "CPU ray tracer (PPM). See DOCUMENTATION.md")]
struct Cli {
    /// Which built-in scene to render
    #[arg(long, value_enum, default_value_t = Preset::Showcase)]
    preset: Preset,

    /// Image width in pixels
    #[arg(long, default_value_t = 320)]
    width: u32,

    /// Image height in pixels
    #[arg(long, default_value_t = 240)]
    height: u32,

    /// Write PPM to this path instead of stdout
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Enable procedural / mapped textures (-t in project brief)
    #[arg(long, default_value_t = false)]
    textures: bool,

    /// Enable reflective materials where defined
    #[arg(long, default_value_t = false)]
    reflection: bool,

    /// Enable refractive (glass / water) shading where defined
    #[arg(long, default_value_t = false)]
    refraction: bool,

    /// Add emissive particle dust to the `all-objects` style scenes
    #[arg(long, default_value_t = false)]
    particles: bool,

    /// Add a wavy fluid heightfield surface (ray-marched)
    #[arg(long, default_value_t = false)]
    fluid: bool,

    /// Write the four 800×600 auditor `.ppm` files into `images/`
    #[arg(long, default_value_t = false)]
    render_auditor: bool,

    /// Launch interactive UI (requires `--features gui`)
    #[arg(long, default_value_t = false)]
    gui: bool,

    /// Multiply scene `brightness` after the preset is built (global exposure)
    #[arg(long)]
    brightness_mul: Option<f32>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    #[cfg(feature = "gui")]
    if cli.gui {
        if let Err(e) = rt::gui::run_gui() {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return Ok(());
    }

    #[cfg(not(feature = "gui"))]
    if cli.gui {
        eprintln!("Rebuild with: cargo run --features gui -- --gui");
        std::process::exit(1);
    }

    if cli.render_auditor {
        fs::create_dir_all("images")?;
        let w = 800u32;
        let h = 600u32;
        let sets = [
            ("images/01_sphere.ppm", Preset::Sphere, RenderSettings {
                textures: true,
                reflection: true,
                ..Default::default()
            }),
            (
                "images/02_plane_cube_dim.ppm",
                Preset::PlaneCube,
                RenderSettings {
                    textures: true,
                    reflection: true,
                    ..Default::default()
                },
            ),
            (
                "images/03_all_primitives.ppm",
                Preset::AllObjects,
                RenderSettings {
                    textures: true,
                    reflection: true,
                    refraction: true,
                    particles: true,
                    fluid: true,
                },
            ),
            (
                "images/04_all_primitives_alt_camera.ppm",
                Preset::AllObjectsAltCam,
                RenderSettings {
                    textures: true,
                    reflection: true,
                    refraction: true,
                    particles: true,
                    fluid: true,
                },
            ),
        ];

        for (path, preset, settings) in sets {
            eprintln!("Rendering {} ...", path);
            let (scene, eff_settings) = build_scene(preset, w, h, &settings, &ObjectPlacement::default());
            let pixels = render_image(&scene, w, h, &eff_settings);
            write_ppm_file(std::path::Path::new(path), &pixels, w, h)?;
        }
        eprintln!("Done. Open the files in `images/` (800×600 P3 PPM).");
        return Ok(());
    }

    let settings = RenderSettings {
        textures: cli.textures,
        reflection: cli.reflection,
        refraction: cli.refraction,
        particles: cli.particles,
        fluid: cli.fluid,
    };

    let (mut scene, eff_settings) =
        build_scene(cli.preset, cli.width, cli.height, &settings, &ObjectPlacement::default());
    if let Some(m) = cli.brightness_mul {
        scene.brightness *= m;
    }
    let pixels = render_image(&scene, cli.width, cli.height, &eff_settings);

    if let Some(path) = &cli.output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_ppm_file(path, &pixels, cli.width, cli.height)?;
    } else {
        write_ppm_p3(&mut stdout().lock(), &pixels, cli.width, cli.height)?;
    }

    Ok(())
}
