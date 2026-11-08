use crate::data::{
    PasswordResetParams, PasswordResetPayload, PasswordResetRequest, VerifyEmailRequest,
};
use serde_json::json;
use validator::Validate;

#[test]
fn reset_request_rejects_weak_passwords() {
    for password in ["short", "onlylowercase", "NoNumbers!", "NoSymbols123"] {
        let request: PasswordResetPayload = serde_json::from_value(json!({
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
        let reset: PasswordResetParams = serde_json::from_value(payload.clone()).unwrap();
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
    let reset: PasswordResetParams = serde_json::from_value(payload.clone()).unwrap();
    let password: PasswordResetPayload = serde_json::from_value(payload.clone()).unwrap();
    let verification: VerifyEmailRequest = serde_json::from_value(payload).unwrap();
    assert!(
        password.validate().is_ok(),
        "Valid reset password cannot reach the service"
    );
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
        serde_json::from_value::<PasswordResetParams>(payload.clone()).is_err(),
        "Reset input lacks the user identity required by token consumption"
    );
    assert!(
        serde_json::from_value::<VerifyEmailRequest>(payload).is_err(),
        "Verification input lacks the user identity required by token consumption"
    );
}

#[test]
fn reset_email_request_rejects_invalid_email() {
    let request: PasswordResetRequest =
        serde_json::from_value(json!({"email": "not-an-email"})).unwrap();
    assert!(
        request.validate().is_err(),
        "Invalid reset email reaches the sender"
    );
}
