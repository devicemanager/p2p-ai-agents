use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub created_at: DateTime<Utc>,
    pub last_rotated_at: Option<DateTime<Utc>>,
    pub rotation_warning_sent: bool,
    pub rotation_required: bool,
}

impl KeyMetadata {
    pub fn new() -> Self {
        Self {
            created_at: Utc::now(),
            last_rotated_at: None,
            rotation_warning_sent: false,
            rotation_required: false,
        }
    }

    pub fn check_rotation_status(&mut self, max_age_days: i64) -> RotationStatus {
        let now = Utc::now();
        let age_days = now.signed_duration_since(self.created_at).num_days();

        if age_days >= max_age_days {
            self.rotation_required = true;
            RotationStatus::Required
        } else if age_days >= (max_age_days - 30) {
            if !self.rotation_warning_sent {
                self.rotation_warning_sent = true;
                RotationStatus::Warning(max_age_days - age_days)
            } else {
                RotationStatus::Ok
            }
        } else {
            RotationStatus::Ok
        }
    }

    pub fn rotate(&mut self) {
        self.last_rotated_at = Some(Utc::now());
        self.created_at = Utc::now();
        self.rotation_required = false;
        self.rotation_warning_sent = false;
    }
}

#[derive(Debug, PartialEq)]
pub enum RotationStatus {
    Ok,
    Warning(i64), // Days remaining
    Required,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_metadata_creation() {
        let metadata = KeyMetadata::new();
        assert!(metadata.last_rotated_at.is_none());
        assert!(!metadata.rotation_required);
    }

    #[test]
    fn test_rotation_check_ok() {
        let mut metadata = KeyMetadata::new();
        assert_eq!(metadata.check_rotation_status(90), RotationStatus::Ok);
    }

    #[test]
    fn test_rotation_check_warning() {
        let mut metadata = KeyMetadata::new();
        // Simulate 61 days old
        metadata.created_at = Utc::now() - chrono::Duration::days(61);

        match metadata.check_rotation_status(90) {
            RotationStatus::Warning(days) => assert_eq!(days, 29),
            _ => panic!("Expected warning"),
        }

        // Warning sent flag should be set
        assert!(metadata.rotation_warning_sent);

        // Subsequent check should be Ok (warning already sent)
        assert_eq!(metadata.check_rotation_status(90), RotationStatus::Ok);
    }

    #[test]
    fn test_rotation_check_required() {
        let mut metadata = KeyMetadata::new();
        // Simulate 91 days old
        metadata.created_at = Utc::now() - chrono::Duration::days(91);

        assert_eq!(metadata.check_rotation_status(90), RotationStatus::Required);
        assert!(metadata.rotation_required);
    }

    #[test]
    fn test_rotate_resets_metadata() {
        let mut metadata = KeyMetadata::new();
        metadata.created_at = Utc::now() - chrono::Duration::days(100);
        metadata.rotation_required = true;
        metadata.rotation_warning_sent = true;

        metadata.rotate();

        assert!(metadata.last_rotated_at.is_some());
        assert!(!metadata.rotation_required);
        assert!(!metadata.rotation_warning_sent);
        // Created at should be recent
        assert!(
            Utc::now()
                .signed_duration_since(metadata.created_at)
                .num_seconds()
                < 5
        );
    }
}
