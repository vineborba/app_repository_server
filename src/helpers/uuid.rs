use uuid::Uuid;
pub fn generate_uuid(tokenless: bool) -> String {
    let uuid = Uuid::new_v4().to_string();
    if tokenless {
        uuid.replace("-", "")
    } else {
        uuid
    }
}
