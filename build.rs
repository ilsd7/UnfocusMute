use std::env;

const APP_ICON_ICO: &str = "assets/app-icon.ico";
const GITHUB_ICON_ICO: &str = "assets/github-invertocat.ico";

fn main() {
    println!("cargo:rerun-if-changed={APP_ICON_ICO}");
    println!("cargo:rerun-if-changed={GITHUB_ICON_ICO}");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }
    if !is_release_profile() && is_cross_compiling() {
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    let resource_version = windows_resource_version();
    resource.set_icon(APP_ICON_ICO);
    resource.set_icon_with_id(GITHUB_ICON_ICO, "2");
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
        let message = format!(
            "failed to compile Windows resources: {error}. Install llvm-rc or build from a Visual Studio Developer Prompt with rc.exe available"
        );
        assert!(!is_release_profile(), "{message}");
        println!("cargo:warning={message}");
    }
}

fn is_release_profile() -> bool {
    env::var("PROFILE").as_deref() == Ok("release")
}

fn is_cross_compiling() -> bool {
    env::var_os("HOST") != env::var_os("TARGET")
}

fn windows_resource_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_owned());
    let core_version = version.split(['-', '+']).next().unwrap_or("0.0.0");
    let mut parts = [0u16; 4];
    for (index, part) in core_version.split('.').take(parts.len()).enumerate() {
        parts[index] = part.parse::<u16>().unwrap_or(0);
    }

    format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3])
}
