use crate::error::ApiError;
use validator::ValidationErrors;

pub fn validation_errors_to_api_error(errors: ValidationErrors) -> ApiError {
    let messages: Vec<String> = errors
        .field_errors() // returns an iterator of tuple (field_name, &Vec<ValidationError>)
        .into_iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |err| {
                let msg = err
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "Validation failed".to_string());
                format!("{}: {}", field, msg)
            })
        })
        .collect();

    ApiError::ValidationError(messages.join("; "))
}

// Works on any type that implements Validate
pub fn validate_dto<T: validator::Validate>(dto: &T) -> Result<(), ApiError> {
    dto.validate().map_err(validation_errors_to_api_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Validate)]
    struct TestDto {
        #[validate(length(min = 3, message = "Name too short"))]
        name: String,
        #[validate(email(message = "Invalid email format"))]
        email: String,
    }

    #[test]
    fn validate_dto_accepts_valid_data() {
        let dto = TestDto {
            name: "John".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(validate_dto(&dto).is_ok());
    }

    #[test]
    fn validate_dto_rejects_invalid_data() {
        let dto = TestDto {
            name: "Jo".to_string(), // too short
            email: "john@example.com".to_string(),
        };
        assert!(validate_dto(&dto).is_err());
    }

    #[test]
    fn validation_error_includes_field_name() {
        let dto = TestDto {
            name: "Jo".to_string(),
            email: "john@example.com".to_string(),
        };
        if let Err(ApiError::ValidationError(msg)) = validate_dto(&dto) {
            assert!(msg.contains("name"), "Error should contain field name");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn validation_error_includes_custom_message() {
        let dto = TestDto {
            name: "Jo".to_string(),
            email: "john@example.com".to_string(),
        };
        if let Err(ApiError::ValidationError(msg)) = validate_dto(&dto) {
            assert!(msg.contains("Name too short"), "Error should contain custom message");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn multiple_errors_joined_with_semicolon() {
        let dto = TestDto {
            name: "Jo".to_string(),            // invalid
            email: "not-an-email".to_string(), // invalid
        };
        if let Err(ApiError::ValidationError(msg)) = validate_dto(&dto) {
            assert!(msg.contains(";"), "Multiple errors should be joined with semicolon");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn validation_error_format_is_field_colon_message() {
        let dto = TestDto {
            name: "Jo".to_string(),
            email: "valid@email.com".to_string(),
        };
        if let Err(ApiError::ValidationError(msg)) = validate_dto(&dto) {
            assert!(msg.contains("name:"), "Format should be 'field: message'");
        } else {
            panic!("Expected ValidationError");
        }
    }
}
