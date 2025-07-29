use ctor::ctor;

include!(concat!(env!("OUT_DIR"), "/exports.rs"));

#[cfg(not(target_family = "windows"))]
compile_error!("Only Support on windows");

#[ctor]
fn __init() {
    unsafe {
        proxy::setup_redirection();
    }

}

pub fn init() {
    unsafe {
        proxy::setup_redirection();
    }
}
