use uuid::Uuid;

use crate::error::ApiError;

pub fn parse_uuid(id: &str, entity_name: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|e| {
        log::warn!("Invalid {} UUID '{}': {}", entity_name, id, e);
        ApiError::ValidationError(format!("Invalid {} ID format", entity_name))
    })
}
