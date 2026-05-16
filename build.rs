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
    icon_dir.add_entry(ico::IconDirEntry::encode_as_png(&image)?);

    let mut ico_file = File::create(&destination)?;
    icon_dir.write(&mut ico_file)?;
    Ok(destination)
}
