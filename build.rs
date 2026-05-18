use std::env;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

const APP_ICON_PNG: &str = "assets/app-icon.png";

fn main() {
    println!("cargo:rerun-if-changed={APP_ICON_PNG}");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    let icon_path = match build_ico_from_png(Path::new(APP_ICON_PNG)) {
        Ok(path) => path,
        Err(error) => {
            println!("cargo:warning=failed to prepare Windows icon from {APP_ICON_PNG}: {error}");
            return;
        }
    };

    let mut resource = winresource::WindowsResource::new();
    let resource_version = windows_resource_version();
    resource.set_icon(icon_path.to_string_lossy().as_ref());
    resource.set("ProductName", "UnfocusMute");
    resource.set("FileDescription", "UnfocusMute");
    resource.set("FileVersion", &resource_version);
    resource.set("ProductVersion", &resource_version);
    resource.set("LegalCopyright", "Apache-2.0");
    let manifest = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="{resource_version}" processorArchitecture="*" name="UnfocusMute" type="win32"/>
  <description>UnfocusMute</description>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#
    );
    resource.set_manifest(&manifest);

    if let Err(error) = resource.compile() {
        let message = format!("failed to compile Windows resources: {error}");
        if env::var("PROFILE").as_deref() == Ok("release") {
            panic!("{message}");
        }
        println!("cargo:warning={message}");
    }
}

fn windows_resource_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_owned());
    let core_version = version.split(['-', '+']).next().unwrap_or("0.0.0");
    let mut parts = core_version
        .split('.')
        .take(4)
        .map(|part| {
            part.parse::<u16>()
                .map(|value| value.to_string())
                .unwrap_or_else(|_| "0".to_owned())
        })
        .collect::<Vec<_>>();

    while parts.len() < 4 {
        parts.push("0".to_owned());
    }

    parts.join(".")
}

fn build_ico_from_png(source: &Path) -> io::Result<PathBuf> {
    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OUT_DIR is not set"))?,
    );
    let destination = out_dir.join("unfocusmute.ico");

    let png = File::open(source)?;
    let image = ico::IconImage::read_png(png)?;
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16, 20, 24, 32, 40, 48, 64, 128, 256] {
        let resized = resize_icon_for_shell(&image, size);
        icon_dir.add_entry(ico::IconDirEntry::encode_as_png(&resized)?);
    }

    let mut ico_file = File::create(&destination)?;
    icon_dir.write(&mut ico_file)?;
    Ok(destination)
}

fn resize_icon_for_shell(source: &ico::IconImage, size: u32) -> ico::IconImage {
    let (left, top, width, height) = alpha_bounds(source);
    let padding = icon_padding(size);
    let target = size.saturating_sub(padding * 2).max(1);
    let scale = (target as f32 / width as f32).min(target as f32 / height as f32);
    let draw_width = ((width as f32 * scale).round() as u32).clamp(1, target);
    let draw_height = ((height as f32 * scale).round() as u32).clamp(1, target);
    let offset_x = (size - draw_width) / 2;
    let offset_y = (size - draw_height) / 2;

    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..draw_height {
        for x in 0..draw_width {
            let src_left = left as f64 + x as f64 * width as f64 / draw_width as f64;
            let src_top = top as f64 + y as f64 * height as f64 / draw_height as f64;
            let src_right = left as f64 + (x + 1) as f64 * width as f64 / draw_width as f64;
            let src_bottom = top as f64 + (y + 1) as f64 * height as f64 / draw_height as f64;
            let dst_index = (((offset_y + y) * size + offset_x + x) * 4) as usize;
            rgba[dst_index..dst_index + 4].copy_from_slice(&sample_area(
                source, src_left, src_top, src_right, src_bottom,
            ));
        }
    }

    ico::IconImage::from_rgba_data(size, size, rgba)
}

fn icon_padding(size: u32) -> u32 {
    match size {
        0..=24 => 1,
        25..=64 => 2,
        65..=128 => 3,
        _ => 4,
    }
}

fn sample_area(image: &ico::IconImage, left: f64, top: f64, right: f64, bottom: f64) -> [u8; 4] {
    let mut total = [0.0; 4];
    let mut weight_total = 0.0;
    let x_start = left.floor().max(0.0) as u32;
    let y_start = top.floor().max(0.0) as u32;
    let x_end = right.ceil().min(image.width() as f64) as u32;
    let y_end = bottom.ceil().min(image.height() as f64) as u32;

    for y in y_start..y_end {
        let y_overlap = overlap(top, bottom, y as f64, y as f64 + 1.0);
        if y_overlap <= 0.0 {
            continue;
        }
        for x in x_start..x_end {
            let x_overlap = overlap(left, right, x as f64, x as f64 + 1.0);
            if x_overlap <= 0.0 {
                continue;
            }
            let weight = x_overlap * y_overlap;
            let pixel = premultiplied_pixel(image, x, y);
            for channel in 0..4 {
                total[channel] += pixel[channel] * weight;
            }
            weight_total += weight;
        }
    }

    if weight_total <= f64::EPSILON {
        return [0, 0, 0, 0];
    }
    for value in &mut total {
        *value /= weight_total;
    }
    unpremultiply(total)
}

fn overlap(left: f64, right: f64, pixel_left: f64, pixel_right: f64) -> f64 {
    right.min(pixel_right) - left.max(pixel_left)
}

fn premultiplied_pixel(image: &ico::IconImage, x: u32, y: u32) -> [f64; 4] {
    let index = ((y * image.width() + x) * 4) as usize;
    let red = image.rgba_data()[index] as f64;
    let green = image.rgba_data()[index + 1] as f64;
    let blue = image.rgba_data()[index + 2] as f64;
    let alpha = image.rgba_data()[index + 3] as f64 / 255.0;
    [red * alpha, green * alpha, blue * alpha, alpha]
}

fn unpremultiply(pixel: [f64; 4]) -> [u8; 4] {
    let alpha = pixel[3].clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return [0, 0, 0, 0];
    }

    [
        (pixel[0] / alpha).round().clamp(0.0, 255.0) as u8,
        (pixel[1] / alpha).round().clamp(0.0, 255.0) as u8,
        (pixel[2] / alpha).round().clamp(0.0, 255.0) as u8,
        (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn alpha_bounds(image: &ico::IconImage) -> (u32, u32, u32, u32) {
    let mut min_x = image.width();
    let mut min_y = image.height();
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let alpha = image.rgba_data()[((y * image.width() + x) * 4 + 3) as usize];
            if alpha > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    if found {
        (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    } else {
        (0, 0, image.width(), image.height())
    }
}
