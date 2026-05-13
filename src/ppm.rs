use std::io::{self, Write};
use std::path::Path;

use crate::color::Color;

pub fn write_ppm_p3<W: Write>(w: &mut W, pixels: &[Color], width: u32, height: u32) -> io::Result<()> {
    writeln!(w, "P3")?;
    writeln!(w, "{} {}", width, height)?;
    writeln!(w, "255")?;
    for c in pixels {
        let [r, g, b] = c.to_u8_rgb();
        writeln!(w, "{} {} {}", r, g, b)?;
    }
    Ok(())
}

pub fn write_ppm_file(path: &Path, pixels: &[Color], width: u32, height: u32) -> io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    write_ppm_p3(&mut f, pixels, width, height)
}
