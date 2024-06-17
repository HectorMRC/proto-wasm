use once_cell::sync::Lazy;
use protobuf::Message;
use std::{env, fs::File, io::Read, path::Path};
use wasmer::{Instance, Module, Store, WasmSlice};
use wasmer_wasix::WasiEnv;

const MEMORY_KEY: &str = "memory";

static PROTO_WASM_BYTES: Lazy<Vec<u8>> = Lazy::new(|| {
    let proto_wasm_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/wasm32-wasi/release/proto_wasm.wasm");

    eprintln!("PROTO_WASM_PATH={:?}", proto_wasm_path);

    let mut f = File::open(proto_wasm_path).expect("wasm file must open");

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)
        .expect("wasm file must be read till its end");

    bytes
});

async fn read_u32_le(mut bytes: &[u8]) -> u32 {
    use wasmer_wasix::virtual_fs::AsyncReadExt;
    bytes
        .read_u32_le()
        .await
        .expect("output len must be readen as a little endian u32")
}

#[tokio::test]
async fn wasm_returning_raw_string() {
    let mut store = Store::default();
    let module = Module::new(&store, &*PROTO_WASM_BYTES).expect("wasm module must be initialized");

    let mut wasi_env = WasiEnv::builder("test")
        .finalize(&mut store)
        .expect("wasi env must be builded");

    let imports = wasi_env
        .import_object(&mut store, &module)
        .expect("wasi import object must not fail");

    let instance =
        Instance::new(&mut store, &module, &imports).expect("instance must be initialized");

    wasi_env
        .initialize(&mut store, instance.clone())
        .expect("instance must be initialized in the wasi environment");

    let new_string = instance
        .exports
        .get_typed_function::<u32, u32>(&store, "new_string")
        .expect("generate wasm function must exists");

    let want = 32 * 1024 * 1024; // 32MB
    let pointer = new_string
        .call(&mut store, want)
        .expect("generate wasm function must be called");

    let memory = instance
        .exports
        .get_memory(MEMORY_KEY)
        .expect("memory must be gotten");

    let view = memory.view(&store);

    let output_len = read_u32_le(
        WasmSlice::new(&view, pointer as u64, 4)
            .expect("wasm slice for output len must be initialized")
            .read_to_vec()
            .expect("wasm slice for output len must be written into a vec")
            .as_slice(),
    )
    .await;

    eprintln!("OUTPUT_LEN={output_len}");

    let output: Vec<u8> = WasmSlice::new(&view, pointer as u64 + 4, output_len as u64)
        .expect("wasm slice for output must be initialized")
        .read_to_vec()
        .expect("wasm slice for output must be written into a vec");

    let got = String::from_utf8_lossy(&output);
    assert_eq!(got, String::from("x").repeat(want as usize));
}

#[tokio::test]
async fn wasm_returning_proto_message() {
    let mut store = Store::default();
    let module = Module::new(&store, &*PROTO_WASM_BYTES).expect("wasm module must be initialized");

    let mut wasi_env = WasiEnv::builder("test")
        .finalize(&mut store)
        .expect("wasi env must be builded");

    let imports = wasi_env
        .import_object(&mut store, &module)
        .expect("wasi import object must not fail");

    let instance =
        Instance::new(&mut store, &module, &imports).expect("instance must be initialized");

    wasi_env
        .initialize(&mut store, instance.clone())
        .expect("instance must be initialized in the wasi environment");

    let new_proto = instance
        .exports
        .get_typed_function::<u32, u32>(&store, "new_proto")
        .expect("generate wasm function must exists");

    let want = 32 * 1024 * 1024; // 32MB

    let pointer = new_proto
        .call(&mut store, want)
        .expect("generate wasm function must be called");

    // needed to close stdout_rx with an EOF.
    wasi_env.on_exit(&mut store, None);

    let memory = instance
        .exports
        .get_memory(MEMORY_KEY)
        .expect("memory must be gotten");

    let view = memory.view(&store);

    let output_len = read_u32_le(
        WasmSlice::new(&view, pointer as u64, 4)
            .expect("wasm slice for output len must be initialized")
            .read_to_vec()
            .expect("wasm slice for output len must be written into a vec")
            .as_slice(),
    )
    .await;

    eprintln!("OUTPUT_LEN={output_len}");

    let output: Vec<u8> = WasmSlice::new(&view, (pointer + 4) as u64, output_len as u64)
        .expect("wasm slice for output must be initialized")
        .read_to_vec()
        .expect("wasm slice for output must be written into a vec");

    let message = proto_wasm::proto::message::Message::parse_from_bytes(&output)
        .expect("proto message must be parsed from bytes");

    assert_eq!(message.value, String::from("x").repeat(want as usize));
}
