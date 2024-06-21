
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .type_attribute(".", "#[derive(serde::Deserialize)]")
        .compile(
            &["./proto/sso.proto", "./proto/notes.proto", "./proto/tags.proto", "./proto/files.proto"],
            &["./proto"],
        )?;

    Ok(())
}
