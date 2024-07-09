
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]")
        .type_attribute("files.CreateFileMetadata.attach_id", "#[derive(Copy)]")
        .field_attribute("auth.RegisterRequest.fingerprint", "#[serde(skip)]")
        .field_attribute("auth.LoginRequest.fingerprint", "#[serde(skip)]")
        .field_attribute("notes.CreateNoteReq.user_id", "#[serde(skip)]")
        .field_attribute("notes.UpdateNoteReq.id", "#[serde(skip)]")
        .field_attribute("notes.UpdateNoteReq.user_id", "#[serde(skip)]")
        .field_attribute("notes.AttachTagReq.user_id", "#[serde(skip)]")
        .field_attribute("notes.AttachTagReq.note_id", "#[serde(skip)]")
        .field_attribute("tags.CreateTagReq.user_id", "#[serde(skip)]")
        .field_attribute("tags.UpdateTagReq.id", "#[serde(skip)]")
        .field_attribute("tags.UpdateTagReq.user_id", "#[serde(skip)]")
        .field_attribute("shelves.UpdateShelfReq.user_id", "#[serde(skip)]")
        .field_attribute("shelves.ConvertToNoteReq.user_id", "#[serde(skip)]")
        .compile(
            &[
                "./proto/sso.proto",
                "./proto/notes.proto",
                "./proto/tags.proto",
                "./proto/files.proto",
                "./proto/shelves.proto",
            ],
            &["./proto"],
        )?;

    Ok(())
}
