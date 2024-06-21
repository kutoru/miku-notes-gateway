
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./proto/sso.proto")?;
    tonic_build::compile_protos("./proto/notes.proto")?;
    Ok(())
}
