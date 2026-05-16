use std::env;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=icon.png");

    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    let icon_path = match build_ico_from_png(Path::new("icon.png")) {
        Ok(path) => path,
        Err(error) => {
            println!("cargo:warning=failed to prepare Windows icon from icon.png: {error}");
            return;
        }
    };

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(icon_path.to_string_lossy().as_ref());
    resource.set("ProductName", "UnfocusMute");
    resource.set("FileDescription", "UnfocusMute");
    resource.set("LegalCopyright", "Apache-2.0");
    resource.set_manifest(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="0.1.0.0" processorArchitecture="*" name="UnfocusMute" type="win32"/>
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
</assembly>"#,
    );

    if let Err(error) = resource.compile() {
        println!("cargo:warning=failed to compile Windows resources: {error}");
    }
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
        icon_dir.add_entry(ico::IconDirEntry::encode(&resized)?);
    }

    let mut ico_file = File::create(&destination)?;
    icon_dir.write(&mut ico_file)?;
    Ok(destination)
}

fn resize_icon_for_shell(source: &ico::IconImage, size: u32) -> ico::IconImage {
    let (left, top, width, height) = alpha_bounds(source);
    let padding = if size <= 24 { 0 } else { (size / 24).min(4) };
    let target = size.saturating_sub(padding * 2).max(1);
    let scale = (target as f32 / width as f32).min(target as f32 / height as f32);
    let draw_width = ((width as f32 * scale).round() as u32).clamp(1, target);
    let draw_height = ((height as f32 * scale).round() as u32).clamp(1, target);
    let offset_x = (size - draw_width) / 2;
    let offset_y = (size - draw_height) / 2;

    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..draw_height {
        for x in 0..draw_width {
            let src_x = left as f32 + (x as f32 + 0.5) * width as f32 / draw_width as f32 - 0.5;
            let src_y = top as f32 + (y as f32 + 0.5) * height as f32 / draw_height as f32 - 0.5;
            let dst_index = (((offset_y + y) * size + offset_x + x) * 4) as usize;
            rgba[dst_index..dst_index + 4].copy_from_slice(&sample_bilinear(source, src_x, src_y));
        }
    }

    ico::IconImage::from_rgba_data(size, size, rgba)
}

fn sample_bilinear(image: &ico::IconImage, x: f32, y: f32) -> [u8; 4] {
    let x0 = x.floor().clamp(0.0, (image.width() - 1) as f32) as u32;
    let y0 = y.floor().clamp(0.0, (image.height() - 1) as f32) as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);
    let wx = x - x.floor();
    let wy = y - y.floor();

    let top = mix_pixel(pixel(image, x0, y0), pixel(image, x1, y0), wx);
    let bottom = mix_pixel(pixel(image, x0, y1), pixel(image, x1, y1), wx);
    unpremultiply(mix_pixel(top, bottom, wy))
}

fn pixel(image: &ico::IconImage, x: u32, y: u32) -> [f32; 4] {
    let index = ((y * image.width() + x) * 4) as usize;
    let red = image.rgba_data()[index] as f32;
    let green = image.rgba_data()[index + 1] as f32;
    let blue = image.rgba_data()[index + 2] as f32;
    let alpha = image.rgba_data()[index + 3] as f32 / 255.0;
    [red * alpha, green * alpha, blue * alpha, alpha]
}

fn mix_pixel(left: [f32; 4], right: [f32; 4], amount: f32) -> [f32; 4] {
    [
        left[0] + (right[0] - left[0]) * amount,
        left[1] + (right[1] - left[1]) * amount,
        left[2] + (right[2] - left[2]) * amount,
        left[3] + (right[3] - left[3]) * amount,
    ]
}

fn unpremultiply(pixel: [f32; 4]) -> [u8; 4] {
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
            if alpha > 8 {
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
