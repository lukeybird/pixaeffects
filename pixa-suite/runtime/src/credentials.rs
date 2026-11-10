//! OS credential storage only. There is no plaintext-file fallback.
use crate::account::DeviceToken;
pub trait CredentialStore: Send + 'static {
    fn load(&mut self) -> Result<Option<DeviceToken>, String>;
    fn save(&mut self, token: &DeviceToken) -> Result<(), String>;
    fn delete(&mut self) -> Result<(), String>;
}
pub struct NativeStore {
    service: String,
}
impl NativeStore {
    pub fn new(product: &str) -> Result<Self, String> {
        if product.is_empty()
            || product.len() > 64
            || !product
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("Invalid account storage identifier.".into());
        }
        Ok(Self {
            service: format!("com.aiwebdesignfirm.{product}"),
        })
    }
    fn entry(&self) -> Result<keyring::Entry, String> {
        keyring::Entry::new(&self.service, "device-access")
            .map_err(|_| "OS account storage is unavailable.".into())
    }
}
impl CredentialStore for NativeStore {
    fn load(&mut self) -> Result<Option<DeviceToken>, String> {
        match self.entry()?.get_password() {
            Ok(value) => DeviceToken::parse(value).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err("Could not read saved sign-in from OS account storage.".into()),
        }
    }
    fn save(&mut self, token: &DeviceToken) -> Result<(), String> {
        self.entry()?
            .set_password(token.secret())
            .map_err(|_| "Could not save sign-in in OS account storage.".into())
    }
    fn delete(&mut self) -> Result<(), String> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(
                "Could not remove saved sign-in. Retry disconnecting before leaving this account."
                    .into(),
            ),
        }
    }
}
