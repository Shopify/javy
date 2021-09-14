mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::{context::Context, value::Value};

use once_cell::sync::OnceCell;
use std::io;
use std::path::PathBuf;
use std::{env, fs};
use transcode::{transcode_input, transcode_output};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn _start() {}

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let input = env::var("JAVY_INPUT").expect("Couldn't read JAVY_INPUT env var");
    let script_name = input.clone();
    unsafe {
        let mut context = Context::default();
        context.register_globals(io::stdout()).unwrap();

        let contents = fs::read_to_string::<PathBuf>(input.into()).unwrap();

        let _ = context.eval_global(&script_name, &contents).unwrap();
        let global = context.global_object().unwrap();
        let shopify = global.get_property("Shopify").unwrap();
        let main = shopify.get_property("main").unwrap();

        JS_CONTEXT.set(context).unwrap();
        ENTRYPOINT.0.set(shopify).unwrap();
        ENTRYPOINT.1.set(main).unwrap();
    }
}

#[export_name = "shopify_main"]
pub extern "C" fn run() {
    unsafe {
        let context = JS_CONTEXT.get().unwrap();
        let shopify = ENTRYPOINT.0.get().unwrap();
        let main = ENTRYPOINT.1.get().unwrap();
        let input_bytes = engine::load().expect("Couldn't load input");

        let input_value = transcode_input(&context, &input_bytes).unwrap();
        let output_value = main.call(&shopify, &[input_value]);

        if output_value.is_err() {
            panic!("{}", output_value.unwrap_err().to_string());
        }

        let output = transcode_output(output_value.unwrap()).unwrap();
        engine::store(&output).expect("Couldn't store output");
    }
}
