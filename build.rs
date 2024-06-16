use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    protobuf_codegen::Codegen::new()
        .protoc()
        .include("src/proto")
        .input("src/proto/message.proto")
        .out_dir("src/proto")
        .run_from_script();

    Ok(())
}
