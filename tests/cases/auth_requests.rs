use crate::data::{ResetPasswordRequest, VerifyEmailRequest};
use serde_json::json;
use validator::Validate;

#[test]
fn reset_request_rejects_weak_passwords() {
    for password in ["short", "onlylowercase", "NoNumbers!", "NoSymbols123"] {
        let request: ResetPasswordRequest = serde_json::from_value(json!({
            "user_id": uuid::Uuid::new_v4(),
            "token": "synthetic-token",
            "new_password": password
        }))
        .unwrap();
        assert!(
            request.validate().is_err(),
            "Reset accepts a password rejected at signup: {password}"
        );
    }
}

#[test]
fn confirmation_requests_reject_empty_or_oversized_tokens() {
    for token in [String::new(), "a".repeat(257)] {
        let payload = json!({
            "user_id": uuid::Uuid::new_v4(), "token": token,
            "new_password": "StrongPassword123!"
        });
        let reset: ResetPasswordRequest = serde_json::from_value(payload.clone()).unwrap();
        let verification: VerifyEmailRequest = serde_json::from_value(payload).unwrap();
        assert!(
            reset.validate().is_err(),
            "Reset accepts an invalid token length"
        );
        assert!(
            verification.validate().is_err(),
            "Verification accepts an invalid token length"
        );
    }
}

#[test]
fn confirmation_requests_accept_valid_inputs() {
    let payload = json!({
        "user_id": uuid::Uuid::new_v4(), "token": "synthetic-token",
        "new_password": "StrongPassword123!"
    });
    let reset: ResetPasswordRequest = serde_json::from_value(payload.clone()).unwrap();
    let verification: VerifyEmailRequest = serde_json::from_value(payload).unwrap();
    assert!(
        reset.validate().is_ok(),
        "Valid reset input cannot reach the service"
    );
    assert!(
        verification.validate().is_ok(),
        "Valid verification input cannot reach the service"
    );
}

#[test]
fn confirmation_requests_require_user_identity() {
    let payload = json!({"token": "synthetic-token", "new_password": "StrongPassword123!"});
    assert!(
        serde_json::from_value::<ResetPasswordRequest>(payload.clone()).is_err(),
        "Reset input lacks the user identity required by token consumption"
    );
    assert!(
        serde_json::from_value::<VerifyEmailRequest>(payload).is_err(),
        "Verification input lacks the user identity required by token consumption"
    );
}
