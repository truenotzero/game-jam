use std::{env, fs::File, path::Path};

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};

extern crate gl_generator;

fn create_gl_bindings() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap()
}

fn main() {
    create_gl_bindings();

    // set env vars
    let mut dir = env::current_dir().unwrap();

    dir.push("res");
    // create_resources(&dir);
    dir.pop();
}
