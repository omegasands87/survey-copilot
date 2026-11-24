use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let manifest_path = out_dir.join("common_controls.manifest");
        let rc_path = out_dir.join("manifest.rc");
        let obj_path = out_dir.join("manifest.o");

        let manifest_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;

        fs::write(&manifest_path, manifest_content).expect("Failed to write manifest");
        let rc_content = format!(
            "1 24 \"{}\"\n",
            manifest_path.to_str().unwrap().replace('\\', "/")
        );
        fs::write(&rc_path, rc_content).expect("Failed to write manifest.rc");

        let status = Command::new("windres")
            .arg("-i")
            .arg(&rc_path)
            .arg("-o")
            .arg(&obj_path)
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!(
                    "cargo:rustc-link-arg={}",
                    obj_path.to_str().unwrap().replace('\\', "/")
                );
            }
        }
    }

    let attrs = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
