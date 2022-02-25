(module
  (import "js_engine" "canonical_abi_realloc" (func $js_canonical_abi_realloc (param i32 i32 i32 i32) (result i32)))
  (import "js_engine" "init_src" (func $js_init_src (param i32 i32)))
  (import "js_engine" "execute" (func $js_execute (param i32 i32 i32)))
  (import "js_engine" "memory" (memory $js_engine_memory 1)) ;; 0
  (memory $js_code_memory 1) ;; 1

  (func (export "shopify_main")
    (local $malloc_result i32)
    (local $js_string_length_bytes i32)

    ;; orignal_ptr: *mut u8, original_size: usize, alignment: usize, new_size: usize    
    ;; start arguments for alloc
    i32.const 0
    i32.const 0
    ;; https://doc.rust-lang.org/reference/type-layout.html
    ;; Alignment of [u8; N] == 1
    i32.const 1
    (local.tee $js_string_length_bytes (i32.const {{ js_string_length_bytes }}))
    (call $js_canonical_abi_realloc)
    (local.tee $malloc_result) ;; Destination address (result of malloc)

    ;; Copy js src to js engine memory
    ;; Address of JS in our memory
    i32.const 1 
    local.get $js_string_length_bytes ;; length of the copy
    memory.copy $js_engine_memory $js_code_memory ;; Copy to imported (0) from our memory (1)

    ;; Initialize the javascript source.
    local.get $malloc_result
    local.get $js_string_length_bytes
    call $js_init_src

    ;; start execute arguments
    ;; We are not providing the optional function name
    ;; so just pass all 0s
    i32.const 0
    i32.const 0
    i32.const 0
    call $js_execute
  )

  ;; This data segments initializes memory 1
  ;; which is JavaScript's module memory
  ;; at offset 1
  (data 1 (i32.const 1) "{{ js_string_bytes }}")
)