use base64::{alphabet, engine, write};
use std::io::Write;

pub fn encode_base64(buffer: &[u8]) -> Result<String, std::io::Error> {
    let engine =
        engine::GeneralPurpose::new(&alphabet::STANDARD, engine::GeneralPurposeConfig::default());
    let mut encoder = write::EncoderStringWriter::new(&engine);
    encoder.write_all(buffer)?;
    Ok(encoder.into_inner())
}
