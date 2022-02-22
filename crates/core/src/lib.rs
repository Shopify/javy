mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::{context::Context, value::Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};
use transcode::{transcode_input, transcode_output};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::copy_nonoverlapping;

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

// mod input {
//     #[export_name = "init-src"]
//     unsafe extern "C" fn __wit_bindgen_init_src(arg0: i32, arg1: i32, ){
//       let len0 = arg1 as usize;
//       <super::Input as Input>::init_src(String::from_utf8(Vec::from_raw_parts(arg0 as *mut _, len0, len0)).unwrap());
//     }
//     pub trait Input {
//       fn init_src(js_src: String,);
//     }
//   }

#[export_name = "init_src"]
pub unsafe extern "C" fn init_src(js_str_ptr: *const u8, js_str_len: usize) {
    // TODO: Who is supposed to own this pointer? Is it the caller who allocated, or this module?
    let js = String::from_utf8(Vec::from_raw_parts(js_str_ptr, js_str_len));

    unsafe {
        let context = JS_CONTEXT.get().unwrap();
        let _ = context.eval_global(SCRIPT_NAME, &js).unwrap();
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
    // what should this return?
    // we discussed passing in the script, and fn name instead of hard coding it
    // having some trouble passing them in as args in a ffi-safe way. Tried String and CString types
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
pub unsafe extern "C" fn canonical_abi_realloc(
    orignal_ptr: *mut u8, original_size: usize, alignment: usize, new_size: usize
) -> *mut std::ffi::c_void {
    // 1. Allocate memory of new_size with alignment.
    // 2. If original_ptr != 0
    //    a. copy min(new_size, original_size) bytes from original_ptr to new memory
    //    b. de-allocate original_ptr
    // 3. return new memory ptr

    // https://doc.rust-lang.org/std/alloc/struct.Layout.html
    // https://doc.rust-lang.org/std/alloc/fn.alloc.html
    assert!(new_size >= original_size);

    let new_mem = alloc(Layout::from_size_align(new_size, alignment)
        .expect("Could not allocate with specified size & alignment"));

    if !original_ptr.is_null() {
        copy_nonoverlapping(original_ptr, new_mem, original_size);
        canonical_abi_free(original_ptr, original_size, alignment);
    }
    new_mem
}

#[export_name = "canonical_abi_free"]
pub unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, size: usize, alignment: usize) {
    dealloc(ptr, Layout::from_size_align(size, alignment))
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