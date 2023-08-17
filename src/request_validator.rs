use crate::connection_handler::ParsedRequest;

pub(crate) struct RequestValidator;

pub enum ValidationError {
    InvalidMethod,
}

pub type Result = std::result::Result<(), ValidationError>;

impl RequestValidator {
    pub fn validate(request: &ParsedRequest) -> Result {
        if !Self::validate_method(request) {
            return Err(ValidationError::InvalidMethod);
        }

        Ok(())
    }

    fn validate_method(request: &ParsedRequest) -> bool {
        let valid_methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"];

        if valid_methods.contains(&request.method.as_str()) {
            true
        } else {
            false
        }
    }
}
