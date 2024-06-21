
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .type_attribute(".", "#[derive(serde::Deserialize)]")
        .field_attribute("notes.CreateNoteReq.user_id", "#[serde(default)]")
        .field_attribute("notes.UpdateNoteReq.user_id", "#[serde(default)]")
        .compile(
            &["./proto/sso.proto", "./proto/notes.proto", "./proto/tags.proto", "./proto/files.proto"],
            &["./proto"],
        )?;

    Ok(())
}
