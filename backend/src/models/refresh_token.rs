use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable, Selectable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct NewRefreshToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        self.expires_at > now && self.revoked_at.is_none()
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_token(expires_at: DateTime<Utc>, revoked_at: Option<DateTime<Utc>>) -> RefreshToken {
        RefreshToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "test_hash".to_string(),
            expires_at,
            created_at: Utc::now(),
            revoked_at,
            last_used_at: None,
        }
    }

    #[test]
    fn is_valid_returns_true_when_not_expired_and_not_revoked() {
        let token = make_token(Utc::now() + Duration::hours(1), None);
        assert!(token.is_valid());
    }

    #[test]
    fn is_valid_returns_false_when_expired() {
        let token = make_token(Utc::now() - Duration::hours(1), None);
        assert!(!token.is_valid());
    }

    #[test]
    fn is_valid_returns_false_when_revoked() {
        let token = make_token(
            Utc::now() + Duration::hours(1),
            Some(Utc::now() - Duration::minutes(5)),
        );
        assert!(!token.is_valid());
    }

    #[test]
    fn is_valid_returns_false_when_both_expired_and_revoked() {
        let token = make_token(
            Utc::now() - Duration::hours(1),
            Some(Utc::now() - Duration::minutes(30)),
        );
        assert!(!token.is_valid());
    }

    #[test]
    fn is_revoked_returns_true_when_revoked_at_set() {
        let token = make_token(Utc::now() + Duration::hours(1), Some(Utc::now()));
        assert!(token.is_revoked());
    }

    #[test]
    fn is_revoked_returns_false_when_revoked_at_none() {
        let token = make_token(Utc::now() + Duration::hours(1), None);
        assert!(!token.is_revoked());
    }
}
