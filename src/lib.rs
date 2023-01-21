use std::{env, fs};

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
