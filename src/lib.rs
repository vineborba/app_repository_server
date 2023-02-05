use base64::{alphabet, engine, write};
use std::{env, fs, io::Write};

pub fn encode_base64(buffer: &[u8]) -> Result<String, std::io::Error> {
    let engine =
        engine::GeneralPurpose::new(&alphabet::STANDARD, engine::GeneralPurposeConfig::default());
    let mut encoder = write::EncoderStringWriter::new(&engine);
    encoder.write_all(buffer)?;
    Ok(encoder.into_inner())
}

pub fn create_file_path(
    project: &String,
    branch: &String,
    identifier: &String,
    extension: &String,
) -> Result<String, std::io::Error> {
    let base_path = env::var("UPLOADS_PATH").expect("Failed to load UPLOADS_PATH");
    let full_path = format!("{base_path}/{project}/{branch}/{identifier}.{extension}",);
    fs::create_dir_all(&full_path)?;
    Ok(full_path)
}
