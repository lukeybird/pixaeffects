//! Existing suite device-pairing protocol. Secrets stay in memory and are never part of Debug output.
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
const API: &str = "https://aiwebdesignfirm.com/api/pixashop/device";
pub const ACTIVATE_URL: &str = "https://aiwebdesignfirm.com/pixashop/activate";
pub const ACCOUNT_URL: &str = "https://aiwebdesignfirm.com/settings";

#[derive(Clone, Default)]
pub struct DeviceToken(String);
impl Drop for DeviceToken {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.0.zeroize();
    }
}
impl std::fmt::Debug for DeviceToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeviceToken([redacted])")
    }
}
impl DeviceToken {
    pub fn parse(value: String) -> Result<Self, String> {
        if value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(value))
        } else {
            Err("Invalid activation token.".into())
        }
    }
    pub(crate) fn secret(&self) -> &str {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    #[serde(default)]
    token: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    access: bool,
    #[serde(default)]
    access_seconds: u64,
}
#[derive(Default, Debug)]
pub struct Account {
    token: DeviceToken,
    code: String,
    lease: Option<Instant>,
    generation: u64,
}
impl Answer {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.token.is_empty() {
            let _ = DeviceToken::parse(self.token.clone())?;
        }
        if !self.code.is_empty()
            && (self.code.len() > 32
                || !self
                    .code
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-'))
        {
            return Err("Invalid activation code.".into());
        }
        Ok(())
    }
    pub(crate) fn issued_token(&self) -> Result<Option<DeviceToken>, String> {
        if self.token.is_empty() {
            Ok(None)
        } else {
            DeviceToken::parse(self.token.clone()).map(Some)
        }
    }
}
impl Account {
    pub fn token(&self) -> DeviceToken {
        self.token.clone()
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn allowed(&self, now: Instant) -> bool {
        self.lease.is_some_and(|until| now < until)
    }
    pub fn apply_checked(
        &mut self,
        generation: u64,
        token: DeviceToken,
        answer: Answer,
        now: Instant,
    ) -> Result<(), String> {
        if generation != self.generation {
            return Err("Ignored an outdated account check.".into());
        }
        if self.token.is_empty() {
            self.token = token;
        }
        self.apply(generation, answer, now)
    }
    pub fn forget(&mut self) {
        self.token = DeviceToken::default();
        self.code.clear();
        self.lease = None;
        self.generation = self.generation.saturating_add(1);
    }
    /// A result from a previous sign-in cannot reactivate a signed-out account.
    pub fn apply(&mut self, generation: u64, answer: Answer, now: Instant) -> Result<(), String> {
        if generation != self.generation {
            return Err("Ignored an outdated account check.".into());
        }
        // A malformed response immediately revokes the lease; never keep a stale grant.
        self.lease = None;
        answer.validate()?;
        if !answer.token.is_empty() {
            self.token = DeviceToken::parse(answer.token)?;
        }
        if !answer.code.is_empty()
            && (answer.code.len() > 32
                || !answer
                    .code
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-'))
        {
            return Err("Invalid activation code.".into());
        }
        self.code = answer.code;
        if answer.access && !self.token.is_empty() && answer.access_seconds > 0 {
            self.lease = now.checked_add(Duration::from_secs(answer.access_seconds.min(300)));
            self.code.clear();
        }
        Ok(())
    }
}
#[derive(Serialize)]
struct Request<'a> {
    action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<&'a str>,
}
/// Only called following a person's account action in the development UI. Tests use decoded fixtures.
pub fn check(token: &DeviceToken) -> Result<Answer, String> {
    let body = Request {
        action: if token.is_empty() { "begin" } else { "check" },
        token: (!token.is_empty()).then_some(token.0.as_str()),
    };
    let agent = ureq::Agent::config_builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                .build(),
        )
        .timeout_global(Some(Duration::from_secs(15)))
        .max_redirects(0)
        .build()
        .new_agent();
    let mut response = agent
        .post(API)
        .send_json(body)
        .map_err(|_| "Account check failed. Check your connection and try again.".to_owned())?;
    response
        .body_mut()
        .with_config()
        .limit(16_384)
        .read_json()
        .map_err(|_| "The account service returned an invalid response.".into())
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    fn answer(access: bool, seconds: u64) -> Answer {
        Answer {
            token: "a".repeat(64),
            code: String::new(),
            access,
            access_seconds: seconds,
        }
    }
    #[test]
    fn monotonic_lease_is_bounded_and_denial_revokes_it() {
        let mut a = Account::default();
        let now = Instant::now();
        a.apply(0, answer(true, u64::MAX), now).unwrap();
        assert!(a.allowed(now + Duration::from_secs(299)));
        assert!(!a.allowed(now + Duration::from_secs(300)));
        a.apply(0, answer(false, 300), now).unwrap();
        assert!(!a.allowed(now));
    }
    #[test]
    fn previous_account_response_cannot_restore_access() {
        let mut a = Account::default();
        a.forget();
        assert!(a.apply(0, answer(true, 300), Instant::now()).is_err());
        assert!(!a.allowed(Instant::now()));
    }
    #[test]
    fn malformed_token_and_code_never_grant_access() {
        let mut a = Account::default();
        let mut response = answer(true, 300);
        response.token = "invalid".into();
        assert!(a.apply(0, response, Instant::now()).is_err());
        let mut response = answer(true, 300);
        response.code = "<script>".into();
        assert!(a.apply(0, response, Instant::now()).is_err());
        assert!(!a.allowed(Instant::now()));
    }
    #[test]
    fn credential_prompt_delay_cannot_extend_a_server_lease() {
        let current = Instant::now();
        let issued = current.checked_sub(Duration::from_secs(600)).unwrap();
        let mut account = Account::default();
        account
            .apply_checked(0, DeviceToken::default(), answer(true, 300), issued)
            .unwrap();
        assert!(!account.allowed(current));
    }
    #[test]
    fn debug_never_prints_token() {
        let secret = "b".repeat(64);
        let token = DeviceToken::parse(secret.clone()).unwrap();
        assert!(!format!("{token:?}").contains(&secret));
    }
}
