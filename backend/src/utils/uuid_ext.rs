use uuid::Uuid;

use crate::error::ApiError;

pub fn parse_uuid(id: &str, entity_name: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|e| {
        log::warn!("Invalid {} UUID '{}': {}", entity_name, id, e);
        ApiError::ValidationError(format!("Invalid {} ID format", entity_name))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uuid_accepts_valid_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str, "user");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), uuid_str);
    }

    #[test]
    fn parse_uuid_rejects_empty_string() {
        let result = parse_uuid("", "product");
        assert!(matches!(result, Err(ApiError::ValidationError(_))));
    }

    #[test]
    fn parse_uuid_rejects_invalid_format() {
        let result = parse_uuid("not-a-uuid", "category");
        assert!(matches!(result, Err(ApiError::ValidationError(_))));
    }

    #[test]
    fn parse_uuid_error_includes_entity_name() {
        let result = parse_uuid("invalid", "employee");
        if let Err(ApiError::ValidationError(msg)) = result {
            assert!(msg.contains("employee"), "Error should mention entity name");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn parse_uuid_rejects_partial_uuid() {
        let result = parse_uuid("550e8400-e29b-41d4", "order");
        assert!(matches!(result, Err(ApiError::ValidationError(_))));
    }
}
