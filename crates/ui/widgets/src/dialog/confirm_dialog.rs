pub mod message;
mod view;

/// Task 039: arama's first destructive-action confirmation. Deliberately
/// generic (title/body/confirm-button label, all caller-supplied and
/// already localized) rather than cache-delete-specific, so the next
/// destructive action can reuse this surface instead of inventing its
/// own. The caller decides what "confirmed" means - this widget only
/// reports `Confirm` or `Cancel`.
#[derive(Clone, Debug)]
pub struct ConfirmDialog {
    title: String,
    body: String,
    confirm_label: String,
}

impl ConfirmDialog {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        confirm_label: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            confirm_label: confirm_label.into(),
        }
    }
}
