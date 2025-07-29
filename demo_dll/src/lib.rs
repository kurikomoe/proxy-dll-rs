use anyhow::Result;
use windows::Win32::{Foundation::HINSTANCE, System::SystemServices::DLL_PROCESS_ATTACH};


fn payload() -> Result<()> {
    proxy_dll::init();
    println!("Hello World in Payload");
    Ok(())
}

#[unsafe(export_name = "DllMain")]
extern "system" fn dll_main(_: HINSTANCE, reason: u32, _reserved: isize) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        payload().ok();
    }

    1
}
