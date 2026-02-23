use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ErrorCode {
    #[serde(rename = "PRESENTATION_DECODE_FAILED")]
    PresentationDecodeFailed,
    #[serde(rename = "PRESENTATION_DESERIALIZE_FAILED")]
    PresentationDeserializeFailed,
    #[serde(rename = "PROOF_VERIFICATION_FAILED")]
    ProofVerificationFailed,
    #[serde(rename = "INVALID_SEMAPHORE_COMMITMENT")]
    InvalidSemaphoreCommitment,
    #[serde(rename = "SIGNATURE_PARSE_FAILED")]
    SignatureParseFailed,
    #[serde(rename = "ADDRESS_RECOVERY_FAILED")]
    AddressRecoveryFailed,
    #[serde(rename = "INVALID_APP_ID")]
    InvalidAppId,
    #[serde(rename = "WRONG_OAUTH_SIGNER")]
    WrongOauthSigner,
    #[serde(rename = "CREDENTIAL_ID_FAILED")]
    CredentialIdFailed,
    #[serde(rename = "VERIFICATION_NOT_FOUND")]
    VerificationNotFound,
    #[serde(rename = "VERIFICATION_CHECK_FAILED")]
    VerificationCheckFailed,
    #[serde(rename = "INVALID_REGISTRY_ADDRESS")]
    InvalidRegistryAddress,
    #[serde(rename = "INVALID_CHAIN_ID")]
    InvalidChainId,
    #[serde(rename = "UNSUPPORTED_CHAIN_ID")]
    UnsupportedChainId,
    #[serde(rename = "INVALID_CREDENTIAL_GROUP_ID")]
    InvalidCredentialGroupId,
    #[serde(rename = "SIGNING_FAILED")]
    SigningFailed,
    #[serde(rename = "INVALID_REQUEST_BODY")]
    InvalidRequestBody,
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::bad_request(ErrorCode::InvalidRequestBody, rejection)
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub success: bool,
    pub errors: Vec<ErrorCode>,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub body: ErrorBody,
}

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl fmt::Display) -> Self {
        Self {
            status,
            body: ErrorBody {
                success: false,
                errors: vec![code],
                message: message.to_string(),
            },
        }
    }

    pub fn bad_request(code: ErrorCode, message: impl fmt::Display) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    pub fn unauthorized(code: ErrorCode, message: impl fmt::Display) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message)
    }

    pub fn internal(code: ErrorCode, message: impl fmt::Display) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, code, message)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status, self.body.message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
