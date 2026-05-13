//! Optional interactive control panel: parameters, PPM export, live image preview.

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use glam::Vec3;

use crate::color::Color;
use crate::ppm::write_ppm_file;
use crate::presets::{build_scene_interactive, ObjectPlacement, Preset, RenderSettings};
use crate::render::render_image;

pub fn run_gui() -> Result<(), String> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1520.0, 860.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rt — ray tracer",
        native_options,
        Box::new(|_cc| Ok(Box::new(RtGui::default()) as Box<dyn eframe::App>)),
    )
    .map_err(|e| e.to_string())
}

struct RenderJob {
    started: Instant,
    rx: mpsc::Receiver<Result<(Vec<Color>, u32, u32), String>>,
    /// `Some` → write PPM after a successful trace; `None` → preview texture only.
    save_path: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CamMode {
    InteractiveOrbit,
    FixedPreset,
}

/// Hash of everything that affects the traced image (for debounced auto-preview).
#[derive(Clone, PartialEq)]
struct PreviewDigest {
    preset: Preset,
    cam_mode: CamMode,
    brightness_mul: f32,
    yaw_deg: f32,
    cam_distance: f32,
    cam_height: f32,
    vfov_deg: f32,
    look_at_y: f32,
    textures: bool,
    reflection: bool,
    refraction: bool,
    particles: bool,
    fluid: bool,
    preview_w: u32,
    preview_h: u32,
    placement: ObjectPlacement,
}

fn slider_vec3(ui: &mut egui::Ui, v: &mut Vec3, span: f32) -> bool {
    let mut c = false;
    ui.horizontal(|ui| {
        c |= ui.add(egui::Slider::new(&mut v.x, -span..=span).text("x")).changed();
        c |= ui.add(egui::Slider::new(&mut v.y, -span..=span).text("y")).changed();
        c |= ui.add(egui::Slider::new(&mut v.z, -span..=span).text("z")).changed();
    });
    c
}

/// Returns true if any placement field changed (for live preview debounce).
fn object_placement_controls(ui: &mut egui::Ui, preset: Preset, p: &mut ObjectPlacement) -> bool {
    let mut changed = false;
    match preset {
        Preset::Sphere => {
            ui.label("Sphere (solo preset)");
            changed |= slider_vec3(ui, &mut p.solo_sphere_center, 4.0);
            changed |= ui
                .add(egui::Slider::new(&mut p.solo_sphere_radius, 0.08..=2.5).text("radius"))
                .changed();
        }
        Preset::PlaneCube => {
            ui.label("Floor + cube");
            changed |= ui
                .add(egui::Slider::new(&mut p.pc_plane_y, -2.5..=1.5).text("floor Y"))
                .changed();
            ui.label("Cube center (xyz)");
            changed |= slider_vec3(ui, &mut p.pc_cube_center, 3.0);
            changed |= ui
                .add(egui::Slider::new(&mut p.pc_cube_half, 0.08..=1.8).text("cube half-size (uniform)"))
                .changed();
        }
        Preset::AllObjects | Preset::AllObjectsAltCam | Preset::Showcase => {
            ui.label("All-objects layout");
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_floor_y, -2.5..=0.5).text("floor Y"))
                .changed();
            ui.separator();
            ui.label("Orange sphere — center");
            changed |= slider_vec3(ui, &mut p.ao_sphere_center, 3.5);
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_sphere_radius, 0.08..=1.6).text("radius"))
                .changed();
            ui.separator();
            ui.label("Green cube — center");
            changed |= slider_vec3(ui, &mut p.ao_cube_center, 3.5);
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_cube_half, 0.08..=1.4).text("half-size (uniform)"))
                .changed();
            ui.separator();
            ui.label("Metal cylinder — center (axis vertical)");
            changed |= slider_vec3(ui, &mut p.ao_cylinder_center, 3.5);
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_cylinder_radius, 0.08..=1.2).text("radius"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_cylinder_half_height, 0.08..=2.0).text("half-height (Y)"))
                .changed();
            ui.separator();
            ui.label("Fluid patch");
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_fluid_base_y, -2.0..=0.5).text("base Y"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_fluid_amplitude, 0.0..=0.35).text("wave amplitude"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_fluid_kx, 0.1..=6.0).text("wave kx"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut p.ao_fluid_kz, 0.1..=6.0).text("wave kz"))
                .changed();
        }
    }
    changed
}

struct RtGui {
    preset: Preset,
    cam_mode: CamMode,
    width: u32,
    height: u32,
    preview_w: u32,
    preview_h: u32,
    brightness_mul: f32,
    yaw_deg: f32,
    cam_distance: f32,
    cam_height: f32,
    vfov_deg: f32,
    look_at_y: f32,
    textures: bool,
    reflection: bool,
    refraction: bool,
    particles: bool,
    fluid: bool,
    output_path: String,
    status: String,
    job: Option<RenderJob>,
    preview_texture: Option<egui::TextureHandle>,
    /// When set, a preview retrace is scheduled after this instant (parameter debounce).
    preview_after: Option<Instant>,
    last_preview_digest: Option<PreviewDigest>,
    auto_preview: bool,
    placement: ObjectPlacement,
}

impl RtGui {
    fn settings(&self) -> RenderSettings {
        RenderSettings {
            textures: self.textures,
            reflection: self.reflection,
            refraction: self.refraction,
            particles: self.particles,
            fluid: self.fluid,
        }
    }

    fn preview_digest(&self) -> PreviewDigest {
        PreviewDigest {
            preset: self.preset,
            cam_mode: self.cam_mode,
            brightness_mul: self.brightness_mul,
            yaw_deg: self.yaw_deg,
            cam_distance: self.cam_distance,
            cam_height: self.cam_height,
            vfov_deg: self.vfov_deg,
            look_at_y: self.look_at_y,
            textures: self.textures,
            reflection: self.reflection,
            refraction: self.refraction,
            particles: self.particles,
            fluid: self.fluid,
            preview_w: self.preview_w,
            preview_h: self.preview_h,
            placement: self.placement.clone(),
        }
    }

    fn upload_preview_texture(&mut self, ctx: &egui::Context, pixels: &[Color], w: u32, h: u32) {
        let wu = w as usize;
        let hu = h as usize;
        if wu == 0 || hu == 0 || pixels.len() != wu * hu {
            return;
        }
        let mut rgba = Vec::with_capacity(wu * hu * 4);
        for c in pixels {
            let [r, g, b] = c.to_u8_rgb();
            rgba.extend_from_slice(&[r, g, b, 255]);
        }
        let img = egui::ColorImage::from_rgba_unmultiplied([wu, hu], &rgba);
        let tex = ctx.load_texture("rt_preview", img, egui::TextureOptions::LINEAR);
        self.preview_texture = Some(tex);
    }

    /// Starts a background trace. `save_path`: write PPM when done; always refreshes the preview texture.
    fn start_render_job(&mut self, width: u32, height: u32, save_path: Option<String>) {
        if self.job.is_some() {
            self.status = "A render is already running — wait or cancel is not supported yet.".to_string();
            return;
        }

        let preset = self.preset;
        let settings = self.settings();
        let brightness_mul = self.brightness_mul;
        let cam_mode = self.cam_mode;
        let look_at = Vec3::new(0.0, self.look_at_y, 0.0);
        let orbit_distance = self.cam_distance;
        let yaw_deg = self.yaw_deg;
        let eye_height = self.cam_height;
        let vfov_deg = self.vfov_deg;

        let (tx, rx) = mpsc::channel();
        let label = if save_path.is_some() {
            format!("Rendering {}×{} to disk…", width, height)
        } else {
            format!("Preview {}×{}…", width, height)
        };
        self.status = label;
        self.job = Some(RenderJob {
            started: Instant::now(),
            rx,
            save_path,
        });

        let placement = self.placement.clone();

        thread::spawn(move || {
            let res: Result<(Vec<Color>, u32, u32), String> = (|| {
                let (mut scene, eff) = if cam_mode == CamMode::InteractiveOrbit {
                    build_scene_interactive(
                        preset,
                        width,
                        height,
                        &settings,
                        look_at,
                        orbit_distance,
                        yaw_deg,
                        eye_height,
                        vfov_deg,
                        &placement,
                    )
                } else {
                    crate::presets::build_scene(
                        preset,
                        width,
                        height,
                        &settings,
                        &placement,
                    )
                };
                scene.brightness *= brightness_mul;
                let pixels = render_image(&scene, width, height, &eff);
                Ok((pixels, width, height))
            })();
            let _ = tx.send(res);
        });
    }

    fn poll_job(&mut self, ctx: &egui::Context) {
        let Some(job) = self.job.as_ref() else {
            return;
        };

        match job.rx.try_recv() {
            Ok(Ok((pixels, w, h))) => {
                let started = job.started;
                let save_path = job.save_path.clone();
                self.upload_preview_texture(ctx, &pixels, w, h);

                if let Some(path_str) = save_path {
                    let path = Path::new(&path_str);
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match write_ppm_file(path, &pixels, w, h) {
                        Ok(()) => {
                            self.status = format!(
                                "Saved {}×{} to {} in {:.2}s (preview updated)",
                                w,
                                h,
                                path_str,
                                started.elapsed().as_secs_f32()
                            );
                        }
                        Err(e) => {
                            self.status = format!("Preview OK; write error: {e}");
                        }
                    }
                } else {
                    self.status = format!(
                        "Preview {}×{} in {:.2}s",
                        w,
                        h,
                        started.elapsed().as_secs_f32()
                    );
                }
                self.job = None;
            }
            Ok(Err(e)) => {
                self.status = format!("Render error: {e}");
                self.job = None;
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(40));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.status = "Render thread stopped unexpectedly.".to_string();
                self.job = None;
            }
        }
    }

    fn maybe_schedule_auto_preview(&mut self) {
        if !self.auto_preview {
            return;
        }
        let d = self.preview_digest();
        if self.last_preview_digest.as_ref() != Some(&d) {
            self.last_preview_digest = Some(d);
            self.preview_after = Some(Instant::now() + Duration::from_millis(420));
        }
    }

    fn try_start_debounced_preview(&mut self) {
        if !self.auto_preview || self.job.is_some() {
            return;
        }
        let Some(t) = self.preview_after else {
            return;
        };
        if Instant::now() < t {
            return;
        }
        self.preview_after = None;
        let w = self.preview_w.clamp(64, 800);
        let h = self.preview_h.clamp(48, 600);
        self.start_render_job(w, h, None);
    }
}

impl Default for RtGui {
    fn default() -> Self {
        Self {
            preset: Preset::Showcase,
            cam_mode: CamMode::InteractiveOrbit,
            width: 480,
            height: 360,
            preview_w: 280,
            preview_h: 210,
            brightness_mul: 1.0,
            yaw_deg: 28.0,
            cam_distance: 6.5,
            cam_height: 2.2,
            vfov_deg: 48.0,
            look_at_y: 0.15,
            textures: true,
            reflection: true,
            refraction: true,
            particles: true,
            fluid: true,
            output_path: "images/gui_preview.ppm".to_string(),
            status: "Ready — adjust controls; preview appears after “Render” or enable “Live preview”.".to_string(),
            job: None,
            preview_texture: None,
            preview_after: None,
            last_preview_digest: None,
            auto_preview: false,
            placement: ObjectPlacement::default(),
        }
    }
}

impl eframe::App for RtGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_job(ctx);
        self.try_start_debounced_preview();

        if self.auto_preview && (self.job.is_some() || self.preview_after.is_some()) {
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(560.0)
            .min_width(440.0)
            .max_width(760.0)
            .show(ctx, |ui| {
                ui.heading("rt — controls");
                ui.label("Preview is on the right (narrower). Drag the splitter between panels to resize.");
                ui.separator();

                ui.label("Preset");
                ui.horizontal_wrapped(|ui| {
                    for p in [
                        Preset::Sphere,
                        Preset::PlaneCube,
                        Preset::AllObjects,
                        Preset::AllObjectsAltCam,
                        Preset::Showcase,
                    ] {
                        if ui.selectable_value(&mut self.preset, p, format!("{p:?}")).changed() {
                            self.placement = ObjectPlacement::for_preset(self.preset);
                            self.last_preview_digest = None;
                            self.maybe_schedule_auto_preview();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut self.cam_mode, CamMode::InteractiveOrbit, "Orbit camera")
                        .changed()
                    {
                        self.maybe_schedule_auto_preview();
                    }
                    if ui
                        .radio_value(&mut self.cam_mode, CamMode::FixedPreset, "Fixed (CLI match)")
                        .changed()
                    {
                        self.maybe_schedule_auto_preview();
                    }
                });

                if ui.add(egui::Slider::new(&mut self.width, 64..=1600).text("export width")).changed() {
                    self.maybe_schedule_auto_preview();
                }
                if ui.add(egui::Slider::new(&mut self.height, 48..=1200).text("export height")).changed() {
                    self.maybe_schedule_auto_preview();
                }
                if ui
                    .add(egui::Slider::new(&mut self.brightness_mul, 0.1..=2.5).text("brightness ×"))
                    .changed()
                {
                    self.maybe_schedule_auto_preview();
                }

                ui.separator();
                ui.label("Live preview (debounced)");
                if ui.checkbox(&mut self.auto_preview, "Auto-update preview while editing").changed() {
                    if self.auto_preview {
                        self.last_preview_digest = None;
                        self.maybe_schedule_auto_preview();
                    } else {
                        self.preview_after = None;
                    }
                }
                ui.label("Uses preview resolution below (not export size). Waits ~420ms after you stop changing values.");
                if ui.add(egui::Slider::new(&mut self.preview_w, 160..=640).text("preview width")).changed() {
                    self.maybe_schedule_auto_preview();
                }
                if ui.add(egui::Slider::new(&mut self.preview_h, 120..=480).text("preview height")).changed() {
                    self.maybe_schedule_auto_preview();
                }

                ui.separator();
                ui.label("Bonuses");
                let mut bonus_changed = false;
                for (label, v) in [
                    ("Textures (-t)", &mut self.textures),
                    ("Reflection", &mut self.reflection),
                    ("Refraction", &mut self.refraction),
                    ("Particles", &mut self.particles),
                    ("Fluid surface", &mut self.fluid),
                ] {
                    if ui.checkbox(v, label).changed() {
                        bonus_changed = true;
                    }
                }
                if bonus_changed {
                    self.maybe_schedule_auto_preview();
                }

                ui.separator();
                egui::CollapsingHeader::new("Objects — move & scale")
                    .default_open(true)
                    .show(ui, |ui| {
                        if object_placement_controls(ui, self.preset, &mut self.placement) {
                            self.maybe_schedule_auto_preview();
                        }
                        ui.small("Changing preset resets placement to defaults. Solo sphere also moves the built-in camera target.");
                    });

                ui.separator();
                ui.label("Camera (orbit presets)");
                if ui
                    .add(egui::Slider::new(&mut self.look_at_y, -1.5..=2.5).text("look-at Y"))
                    .changed()
                {
                    self.maybe_schedule_auto_preview();
                }
                if ui
                    .add(egui::Slider::new(&mut self.cam_distance, 2.0..=14.0).text("orbit distance"))
                    .changed()
                {
                    self.maybe_schedule_auto_preview();
                }
                if ui
                    .add(egui::Slider::new(&mut self.yaw_deg, -180.0..=180.0).text("yaw (deg)"))
                    .changed()
                {
                    self.maybe_schedule_auto_preview();
                }
                if ui
                    .add(egui::Slider::new(&mut self.cam_height, 0.5..=5.5).text("eye height"))
                    .changed()
                {
                    self.maybe_schedule_auto_preview();
                }
                if ui
                    .add(egui::Slider::new(&mut self.vfov_deg, 20.0..=80.0).text("vertical FOV"))
                    .changed()
                {
                    self.maybe_schedule_auto_preview();
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("PPM path");
                    ui.text_edit_singleline(&mut self.output_path);
                });

                let busy = self.job.is_some();
                ui.add_enabled_ui(!busy, |ui| {
                    if ui.button("Render export → PPM").clicked() {
                        self.start_render_job(self.width, self.height, Some(self.output_path.clone()));
                    }
                });

                ui.separator();
                ui.label(&self.status);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Preview");
            if let Some(tex) = &self.preview_texture {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Keep the preview visually smaller than the remaining area.
                        const PREVIEW_MAX_W: f32 = 380.0;
                        let display_w = ui.available_width().min(PREVIEW_MAX_W).max(120.0);
                        ui.add(egui::Image::from_texture(tex).max_width(display_w));
                    });
            } else {
                ui.label("No image yet — click “Render export → PPM” or enable “Auto-update preview”.");
            }
        });
    }
}
