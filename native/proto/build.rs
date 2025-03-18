use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("service_descriptor.bin"))
        .compile(
            &[
                "service/v1/service.proto",
                "gossamer/v1/gossamer.proto",
                "application/v1/application.proto",
            ],
            &["proto"],
        )
        .unwrap();
    Ok(())
}
