use keyring::Entry;

pub struct AuthManager {}

impl AuthManager {
    pub fn save_credentials(service: &str, username: &str, password: &str) -> Result<(), String> {
        let entry = Entry::new(service, username).map_err(|e| e.to_string())?;
        entry.set_password(password).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_password(service: &str, username: &str) -> Result<String, String> {
        let entry = Entry::new(service, username).map_err(|e| e.to_string())?;
        entry.get_password().map_err(|e| e.to_string())
    }

    pub fn delete_credentials(service: &str, username: &str) -> Result<(), String> {
        let entry = Entry::new(service, username).map_err(|e| e.to_string())?;
        entry
            .delete_credential()
            .map_err(|e: keyring::Error| e.to_string())
    }
}
