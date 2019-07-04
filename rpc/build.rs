extern crate protobuf_codegen_pure;
// extern crate protoc_rust_grpc;

fn main() {
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src",
        includes: &["../proto"],
        input: &["../proto/discover.proto"],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
        // rust_protobuf: true, // also generate protobuf messages, not just services
        // ..Default::default()
    }).expect("protoc-rust-grpc");
}
