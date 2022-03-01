use anyhow::Result;
use std::io::{copy, stdin, stdout, Write};

#[cfg(not(feature = "standalone-wasi"))]
#[link(wasm_import_module = "shopify_v1")]
extern "C" {
    pub fn input_len(len: *const usize) -> u32;
    pub fn input_copy(buffer: *mut u8) -> u32;
    pub fn output_copy(buffer: *const u8, len: usize) -> u32;
}

pub fn load() -> Result<Vec<u8>> {
    let mut reader = stdin();
    let mut output: Vec<u8> = vec![];

    copy(&mut reader, &mut output)?;
    Ok(output)
}

fn load_from_stdin() -> Result<Vec<u8>> {
    let mut reader = stdin();
    let mut writer: Vec<u8> = vec![];
    copy(&mut reader, &mut writer)?;
    let value: serde_json::Value = serde_json::from_slice(&writer)?;
    rmp_serde::to_vec(&value).map_err(Into::into)
}

#[cfg(not(feature = "standalone-wasi"))]
fn load_from_abi() -> Result<Vec<u8>> {
    let len = 0;
    unsafe {
        input_len(&len);
    }

    let mut input_buffer = vec![0; len];
    unsafe {
        input_copy(input_buffer.as_mut_ptr());
    }

    Ok(input_buffer)
}

pub fn store(bytes: &[u8]) -> Result<()> {
    let mut handle = stdout();
    handle.write_all(bytes)?;

    Ok(())
}

#[cfg(not(feature = "standalone-wasi"))]
unsafe fn store_to_abi(bytes: &[u8]) -> Result<()> {
    output_copy(bytes.as_ptr(), bytes.len());
    Ok(())
}

pub fn store_to_stdout(bytes: &[u8]) -> Result<()> {
    let value: serde_json::Value = rmp_serde::from_read_ref(bytes)?;
    let string = serde_json::to_string(&value)?;

    let mut handle = stdout();
    handle.write_all(string.as_bytes())?;
    writeln!(handle,).unwrap();
    Ok(())
}
