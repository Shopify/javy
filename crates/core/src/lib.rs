mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::{context::Context, value::Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};
use transcode::{transcode_input, transcode_output};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());
static SCRIPT_NAME: &str = "script.js";

// // TODO
// //
// // AOT validations:
// //  1. Ensure that the required exports are present
// //  2. If not present just evaluate the top level statement (?)

// #[export_name = "wizer.initialize"]
// pub extern "C" fn init() {
//     unsafe {
//         let mut context = Context::default();
//         context.register_globals(io::stdout()).unwrap();

//         let mut contents = String::new();
//         io::stdin().read_to_string(&mut contents).unwrap();

//         let _ = context.eval_global(SCRIPT_NAME, &contents).unwrap();
//         let global = context.global_object().unwrap();
//         let shopify = global.get_property("Shopify").unwrap();
//         let main = shopify.get_property("main").unwrap();

//         JS_CONTEXT.set(context).unwrap();
//         ENTRYPOINT.0.set(shopify).unwrap();
//         ENTRYPOINT.1.set(main).unwrap();
//     }
// }

// We want to first wizer this part, then wiser the part with init_src.
// Not sure how to specify both functions for wiser to target since we can only export one with 
// wizer.initialize as the name
#[export_name = "init_engine"]
pub extern "C" fn init_engine() {
    // What does this need to return?
    unsafe {
        let mut context = Context::default();
        context.register_globals(io::stdout()).unwrap();
        JS_CONTEXT.set(context).unwrap();
    }
}

#[export_name = "init_src"]
pub extern "C" fn init_src() {
    // what should this return?
    // we discussed passing in the script, and fn name instead of hard coding it
    // having some trouble passing them in as args in a ffi-safe way. Tried String and CString types
    unsafe {
        let context = JS_CONTEXT.get().unwrap();
        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents).unwrap();

        let _ = context.eval_global(SCRIPT_NAME, &contents).unwrap();
        let global = context.global_object().unwrap();
        let shopify = global.get_property("Shopify").unwrap();
        let main = shopify.get_property("main").unwrap();

        ENTRYPOINT.0.set(shopify).unwrap();
        ENTRYPOINT.1.set(main).unwrap();
    }
}

#[export_name = "execute"]
pub extern "C" fn execute() {
    // what should this return?
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

#[export_name = "canonical_abi_realloc"]
pub extern "C" fn canonical_abi_realloc(_ptr: u32, size: u32, _align: u32, _new_size: u32) -> *mut std::ffi::c_void {
    // Don't really understand what this and the return type is
    Box::into_raw(vec![0u8; size as usize].into_boxed_slice()) as _
}

#[export_name = "canonical_abi_free"]
pub extern "C" fn canonical_abi_free(_ptr: u32, _size: u32, _align: u32) {
}

// #[export_name = "core_malloc"]
// pub extern "C" fn exported_malloc(size: usize) -> *mut std::ffi::c_void {
//     // Leak the vec<u8>, transfering ownership to the caller.
//     // TODO: Consider not zeroing memory (with_capacity & set_len before into_raw_parts).
//     Box::into_raw(vec![0u8; size].into_boxed_slice()) as _
// }

// #[export_name = "run_js_script"]
// pub extern "C" fn run(ptr: *const u8, len: usize) {
//     let (context, js_str) = unsafe {
//         let js_str: &[u8] = std::slice::from_raw_parts(ptr as *const u8, len);
//         let js_str = std::str::from_utf8_unchecked(js_str);

//         (JS_CONTEXT.get().unwrap(), js_str)
//     };
//     let _ = context.eval_global(SCRIPT_NAME, js_str).unwrap();
//     let global = context.global_object().unwrap();
//     let shopify = global.get_property("Shopify").unwrap();
//     let main = shopify.get_property("main").unwrap();

//     let input_bytes = engine::load().expect("Couldn't load input");
//     let input_value = transcode_input(context, &input_bytes).unwrap();
//     let output_value = main.call(&shopify, &[input_value]);

//     if output_value.is_err() {
//         panic!("{}", output_value.unwrap_err().to_string());
//     }

//     let output = transcode_output(output_value.unwrap()).unwrap();
//     engine::store(&output).expect("Couldn't store output");
// }