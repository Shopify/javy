```
// init-engine: function() -> u32
// init-src: function(js-str: string) -> u32
// eval: function(js-func-name: string) -> string
// ENGINE.wasm -> Wizer (Init Engine, Init Context) -> INIT_ENGINE.wasm
// JS_CODE.wasm + INIT_ENGINE.wasm -> Wizer -> INIT_JS_CODE.wasm (Mem snapshot on top of INIT_ENGINE.wasm)


// namespace "shopvm-std-v1-js" "<function_names>"

// ---
// INTERNAL
// (1) Init Stages
// - Initialize Engine
// - Initialize Context
init-engine: function()


// ---
// EXTERNAL
//
// (2) Provide the JS source code to the engine
// - Copy JS source code to engine memory
// - Parse JS code into bytecode
// - Execute JS code to create functions, globals, etc.
init-src: function(js-src: string)


// (3) Execute the previously provided JS source code
// - Read Input from STDIN
// - Parse input from msgpack
// - Call "shopify.main" or provided named object path to function
//   with input and get output
// - Serialize output to msgpack
// - Write output to STDOUT
execute: function(name: option<string>)

// (4) Implied exports
// Implied meaning - they’re implied by this being `.wit` and using interface types. So they don’t get explicitly listed in the `.wit`
// - canonical ABI
// canonical-abi-realloc(u32, u32, u32, u32) 
// canonical-abi-free(u32)


// WASM WASI API:
// - Shouldn't make any calls to host imports
// - Shouldn't be called from your _start
// - We may optimize this for you in the future.
// - Consider this internal, don't document it.
// shopify-init: function()

// _start: function()