use std::{env, fs, io};

pub fn create_file_path(
    project: &String,
    branch: &String,
    identifier: &String,
    extension: &String,
) -> Result<String, std::io::Error> {
    let base_path = env::var("UPLOADS_PATH").expect("Failed to load UPLOADS_PATH");
    let path = format!("{base_path}/{project}/{branch}",);
    fs::create_dir_all(&path)?;
    let full_path = format!("{path}/{identifier}.{extension}");
    Ok(full_path)
}

pub fn write_file_to_disk(path: &String, file_buffer: &[u8]) -> io::Result<()> {
    fs::write(path, file_buffer)
}

pub fn parse_plist_template(
    artifact_id: &str,
    bundle_identifier: &str,
    bundle_version: &str,
    app_name: &str,
) -> String {
    let server_public_url = env::var("PUBLIC_URL").expect("PUBLIC_URL is not set");
    let url = format!("{server_public_url}/artifacts/{artifact_id}/download");
    format!("
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
  <key>items</key>
  <array>
    <dict>
      <key>assets</key>
      <array>
        <dict>
          <key>kind</key>
          <string>software-package</string>
          <key>url</key>
          <string>{url}</string>
        </dict>
      </array>
      <key>metadata</key>
      <dict>
        <key>bundle-identifier</key>
        <string>{bundle_identifier}</string>
        <key>bundle-version</key>
        <string>{bundle_version}</string>
        <key>kind</key>
        <string>software</string>
        <key>title</key>
        <string>{app_name}</string>
      </dict>
    </dict>
  </array>
</dict>
</plist>
")
}

pub fn create_itms_service_url(artifact_id: String) -> String {
    let server_public_url = env::var("PUBLIC_URL").expect("PUBLIC_URL is not set");
    let plist_url = format!("{server_public_url}/artifacts/{artifact_id}/ios-plist");
    format!("itms-services://?action=download-manifest&amp;url={plist_url}")
}

pub fn create_file_url(artifact_id: String) -> String {
    let server_public_url = env::var("PUBLIC_URL").expect("PUBLIC_URL is not set");
    format!("{server_public_url}/artifacts/{artifact_id}/file")
}
