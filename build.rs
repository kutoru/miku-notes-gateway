
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .type_attribute(".", "#[derive(serde::Deserialize)]")
        .field_attribute("auth.RegisterRequest.fingerprint", "#[serde(default)]")
        .field_attribute("auth.LoginRequest.fingerprint", "#[serde(default)]")
        .field_attribute("notes.CreateNoteReq.user_id", "#[serde(default)]")
        .field_attribute("notes.UpdateNoteReq.id", "#[serde(default)]")
        .field_attribute("notes.UpdateNoteReq.user_id", "#[serde(default)]")
        .compile(
            &[
                "./proto/sso.proto",
                "./proto/notes.proto",
                "./proto/tags.proto",
                "./proto/files.proto",
            ],
            &["./proto"],
        )?;

    Ok(())
}
