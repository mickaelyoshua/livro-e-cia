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

// Works on ant type that implements Validate
pub fn validate_dto<T: validator::Validate>(dto: &T) -> Result<(), ApiError> {
    dto.validate().map_err(validation_errors_to_api_error)
}
