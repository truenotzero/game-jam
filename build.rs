use std::{env, fs::File, path::Path};

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use winres::WindowsResource;

extern crate gl_generator;

fn create_gl_bindings() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap()
}

fn set_icon_windows() {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("res/thumbnail.ico")
            .compile()
            .unwrap()
        ;
    }
}

fn main() {
    create_gl_bindings();
    set_icon_windows();
}
