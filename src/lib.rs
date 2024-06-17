use protobuf::Message;

pub mod proto;

#[no_mangle]
pub fn new_string(len: u32) -> *const u8 {
    let output = {
        let output_bytes = String::from("x").repeat(len as usize).into_bytes();
        let output_len = (output_bytes.len() as u32).to_le_bytes();

        [&output_len[..], &output_bytes].concat()
    };

    output.as_ptr()
}

#[no_mangle]
pub fn new_proto(len: u32) -> *const u8 {
    let output = {
        // scope is needed to make sure remaining data is released before
        // returning the output's pointer. Otherwise heap error may occur.
        let output_bytes = proto::message::Message {
            value: String::from("x").repeat(len as usize),
            ..Default::default()
        }
        .write_to_bytes()
        .expect("proto message must write to bytes");

        let output_len = (output_bytes.len() as u32).to_le_bytes();
        [&output_len[..], &output_bytes].concat()
    };

    output.as_ptr()
}
