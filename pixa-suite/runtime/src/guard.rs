//! Subscription-preview policy. An unimplemented recovery adapter always blocks protected editing.
use std::time::Instant;
/// A captured grant for independently repainted native child windows.
/// Checking the deadline here prevents a hidden main window from extending access.
#[derive(Clone, Copy, Debug)]
pub struct ViewportPermit {
    development: bool,
    deadline: Option<Instant>,
}
impl ViewportPermit {
    pub fn allows(&self, now: Instant) -> bool {
        self.development || self.deadline.is_some_and(|until| now < until)
    }
}
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
    pub fn viewport_permit(&self, deadline: Option<Instant>) -> ViewportPermit {
        ViewportPermit {
            development: self.mode == BuildMode::Development,
            deadline: if self.mode == BuildMode::Subscription && self.recovery_ready {
                deadline
            } else {
                None
            },
        }
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
    fn child_window_cannot_extend_an_expired_or_unconfigured_grant() {
        use std::time::Duration;
        let now = Instant::now();
        let until = now + Duration::from_secs(2);
        let permit = Guard::new(BuildMode::Subscription, true).viewport_permit(Some(until));
        assert!(permit.allows(now));
        assert!(!permit.allows(until));
        assert!(
            !Guard::new(BuildMode::Subscription, false)
                .viewport_permit(Some(until))
                .allows(now)
        );
        assert!(
            !Guard::new(BuildMode::Subscription, true)
                .viewport_permit(None)
                .allows(now)
        );
        assert!(
            !Guard::new(BuildMode::Invalid, true)
                .viewport_permit(Some(until))
                .allows(now)
        );
        assert!(
            Guard::new(BuildMode::Development, false)
                .viewport_permit(None)
                .allows(until)
        );
    }
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
