//! Subscription-preview policy. An unimplemented recovery adapter always blocks protected editing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMode {
    Development,
    Subscription,
    Invalid,
}
impl BuildMode {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            None | Some("development") => Self::Development,
            Some("subscription") => Self::Subscription,
            _ => Self::Invalid,
        }
    }
}
pub struct Guard {
    mode: BuildMode,
    recovery_ready: bool,
    used: bool,
    recovery_saved: bool,
}
impl Guard {
    pub fn new(mode: BuildMode, recovery_ready: bool) -> Self {
        Self {
            mode,
            recovery_ready,
            used: false,
            recovery_saved: false,
        }
    }
    pub fn can_edit(&self, access: bool) -> bool {
        self.mode == BuildMode::Development
            || (self.mode == BuildMode::Subscription && self.recovery_ready && access)
    }
    pub fn configuration_valid(&self) -> bool {
        self.mode != BuildMode::Invalid
    }
    pub fn needs_close_review(&self, access: bool) -> bool {
        !self.can_edit(access) && self.used
    }
    pub fn recovery_saved(&self) -> bool {
        self.recovery_saved
    }
    pub fn adapter_ready(&self) -> bool {
        self.recovery_ready
    }
    pub fn note_editor_used(&mut self, access: bool) {
        if self.can_edit(access) {
            self.used = true;
            self.recovery_saved = false;
        }
    }
    pub fn needs_recovery(&self, access: bool) -> bool {
        !self.can_edit(access) && self.used && !self.recovery_saved
    }
    pub fn recovery_succeeded(&mut self) {
        self.recovery_saved = true;
    }
    pub fn may_close(&self, access: bool, discard_confirmed: bool) -> bool {
        !self.needs_close_review(access) || discard_confirmed
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn protected_editing_requires_both_access_and_adapter() {
        assert!(!Guard::new(BuildMode::Subscription, false).can_edit(true));
        assert!(!Guard::new(BuildMode::Subscription, true).can_edit(false));
        assert!(Guard::new(BuildMode::Subscription, true).can_edit(true));
        assert!(!Guard::new(BuildMode::Invalid, true).can_edit(true));
    }
    #[test]
    fn expired_access_keeps_work_open_until_recovery_or_explicit_discard() {
        let mut guard = Guard::new(BuildMode::Subscription, true);
        guard.note_editor_used(true);
        assert!(guard.needs_recovery(false));
        assert!(!guard.may_close(false, false));
        assert!(guard.may_close(false, true));
        guard.recovery_succeeded();
        assert!(!guard.may_close(false, false));
        assert!(guard.may_close(false, true));
        guard.note_editor_used(true);
        assert!(!guard.may_close(false, false));
    }
    #[test]
    fn unopened_editor_can_close_and_invalid_modes_never_become_development() {
        assert!(Guard::new(BuildMode::Subscription, true).may_close(false, false));
        assert_eq!(BuildMode::parse(Some("typo")), BuildMode::Invalid);
        assert_eq!(BuildMode::parse(None), BuildMode::Development);
    }
}
