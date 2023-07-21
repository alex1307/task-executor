use log::info;

fn main() {
    configure_log4rs();
    info!("Building protobufs");
    protobuf_codegen_pure::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["protos/example.proto"])
        .include("protos")
        .run()
        .expect("Codegen failed.");
}