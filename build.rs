#![allow(unused_variables, dead_code)]

use anyhow::Result;
use anyhow::anyhow;
use object::Object;
use serde_json::json;
use std::path::PathBuf;
use std::sync::LazyLock;
use widestring::U16CStr;
use widestring::U16CString;
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::core::PCWSTR;

use handlebars::Handlebars;

/// Target dll to be proxied
static TARGET_DLL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("PROXY_DLL_TARGET").unwrap_or_else(|_| "version.dll".to_string())
});

static TARGET_DLL_PATH: LazyLock<Option<String>> =
    LazyLock::new(|| std::env::var("PROXY_DLL_TARGET_PATH").ok());

/// Should the target dll be override, aka version.dll && version_orig.dll
static IS_OVERRIDE: LazyLock<bool> = LazyLock::new(|| {
    matches!(
        std::env::var("PROXY_DLL_USE_ORIG")
            .unwrap_or_else(|_| "0".to_string())
            .to_lowercase()
            .as_str(),
        "on" | "true" | "1"
    )
});

struct Hbs {
    name: String,
    hbs: String,
    out: PathBuf,
}

impl Hbs {
    pub fn new(name: &str, path: PathBuf) -> Self {
        let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        let hbs = std::fs::read_to_string(&path).expect("Read hbs failed");

        Self {
            name: name.to_owned(),
            hbs,
            out: out_dir.join(name),
        }
    }

    pub fn dump(&self, data: &serde_json::Value) -> Result<String> {
        let reg = Handlebars::new();
        let ret = reg.render_template(&self.hbs, data)?;
        dbg!("{} writes to {}", &self.name, &self.out);
        std::fs::write(&self.out, &ret)?;
        Ok(ret)
    }
}

fn locate_target(target: &str) -> Result<PathBuf> {
    let target_u16 = U16CString::from_str(target)?;

    let path = unsafe {
        let hwnd =
            LoadLibraryW(PCWSTR(target_u16.as_ptr())).map_err(|_| anyhow!("target not found"))?;
        let mut buf = [0u16; 255];
        let sz = GetModuleFileNameW(Some(hwnd), &mut buf);
        let path = U16CStr::from_ptr(buf.as_ptr(), sz as usize)?.to_string()?;
        FreeLibrary(hwnd)?;
        path
    };

    Ok(PathBuf::from(path))
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed=PROXY_DLL_TARGET");
    println!("cargo:rerun-if-env-changed=PROXY_DLL_TARGET_PATH");
    println!("cargo:rerun-if-env-changed=PROXY_DLL_USE_ORIG");

    let def_hbs = Hbs::new("exports.def", PathBuf::from("assets/exports_def.hbs"));
    println!("cargo::rerun-if-changed=assets/exports_def.hbs");
    let rs_hbs = Hbs::new("exports.rs", PathBuf::from("assets/exports_rs.hbs"));
    println!("cargo::rerun-if-changed=assets/exports_rs.hbs");

    let target_dll_name = TARGET_DLL.clone();
    println!("cargo:rustc-cdylib-link-arg=/OUT:{}", &target_dll_name);
    println!("cargo::warning=target dll: {}", &target_dll_name);

    let target_dll_path = match &*TARGET_DLL_PATH {
        Some(path) => PathBuf::from(path),
        None => locate_target(&target_dll_name).expect("target dll not found"),
    };
    let image = std::fs::read(&target_dll_path)?;
    let pe = object::File::parse(&*image)?;

    let exports: Vec<serde_json::Value> = pe
        .exports()?
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            json!({
                "ord": idx+1,
                "name": std::str::from_utf8(v.name()).unwrap(),
            })
        })
        .collect();

    dbg!(target_dll_path.to_str().unwrap().to_lowercase());
    println!(
        "cargo::warning=MESSAGE=target dll path: {}",
        target_dll_path.display()
    );
    let target_dll_real_path = if TARGET_DLL_PATH.is_some() {
        // Use the user provided dll
        target_dll_path
    } else if *IS_OVERRIDE {
        // use the _orig dll
        let filename = PathBuf::from(&*TARGET_DLL);
        PathBuf::from(format!(
            "{}_orig.{}",
            filename.file_stem().unwrap().to_str().unwrap(),
            filename.extension().unwrap().to_str().unwrap()
        ))
    } else if target_dll_path
        .to_str()
        .unwrap()
        .to_lowercase()
        .starts_with(r#"c:\windows"#)
    {
        // Use the found dll in system path
        target_dll_path.clone()
    } else {
        println!(
            "cargo::error=Please provide the dll path, without the path, the loader will recursive infinitely."
        );
        panic!("");
    };

    let data = json!({
        "dll_name": &target_dll_name,
        "dll_path": &target_dll_real_path,
        "func_names": exports,
    });
    dbg!(&data);
    def_hbs.dump(&data)?;
    rs_hbs.dump(&data)?;

    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def_hbs.out.display());

    Ok(())
}
