# rt

CPU **ray tracer** in Rust: spheres, axis-aligned cubes, planes, capped cylinders, shadows, movable camera, **P3 PPM** output, and optional bonuses (textures, reflection, refraction, particles, fluid surface). Optional **egui** UI with live preview and object placement sliders.

| | |
| --- | --- |
| **Full guide** | **[DOCUMENTATION.md](DOCUMENTATION.md)** — CLI, flags, `ObjectPlacement`, code examples |
| **Build** | `cargo build --release` |
| **GUI** | `cargo run --release --features gui -- --gui` |

---

## Quick commands

```bash
cargo build --release
```

```bash
# Four auditor scenes → images/ (800×600 PPM)
cargo run --release -- --render-auditor
```

```bash
# Single image to a file
cargo run --release -- --preset showcase --width 400 --height 300 -o images/out.ppm
```

```bash
# Interactive UI (preview + object sliders)
cargo run --release --features gui -- --gui
```

```bash
# PPM to stdout (redirect to a file)
cargo run --release -- --preset sphere --width 320 --height 240 > preview.ppm
```

---

## Generated outputs (`images/`)

These files are **not** required in git; they appear after you render. Typical layout:

| File | Source | Notes |
| --- | --- | --- |
| `images/01_sphere.ppm` | `--render-auditor` | Single sphere, brighter lighting |
| `images/02_plane_cube_dim.ppm` | `--render-auditor` | Floor + cube, dimmer scene |
| `images/03_all_primitives.ppm` | `--render-auditor` | All shapes + bonuses |
| `images/04_all_primitives_alt_camera.ppm` | `--render-auditor` | Same as above, different camera |
| `images/gui_preview.ppm` | GUI default **Render export → PPM** path | Overwritten each time you export from the UI |

**Viewing PPM:** open in GIMP, IrfanView, ImageGlass, or VS Code with a PPM-capable extension.

**GitHub README thumbnails:** GitHub does not inline **PPM** images. If you want pictures in this README, convert once (example with [ImageMagick](https://imagemagick.org/)):

```bash
magick images/01_sphere.ppm images/01_sphere.png
```

Then you can add:

```markdown
![Sphere preset](images/01_sphere.png)
```

---

## Repo layout (short)

| Path | Role |
| --- | --- |
| `src/` | Library + CLI + optional `gui.rs` |
| `src/presets.rs` | Scenes, `ObjectPlacement`, auditor defaults |
| `images/` | Default output folder (may contain only `.gitkeep` until you render) |
| `DOCUMENTATION.md` | Auditor-oriented usage and API snippets |

---

## License

See [LICENSE](LICENSE).
