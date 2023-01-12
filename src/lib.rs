use std::fs;

pub fn create_file_path(
    project: &String,
    branch: &String,
    identifier: &String,
    extension: &String,
) -> Result<String, std::io::Error> {
    let full_path = format!(
        "../artifacts/{}/{}/{}.{}",
        project, branch, identifier, extension
    );
    fs::create_dir_all(&full_path)?;
    Ok(full_path)
}
