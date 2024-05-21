use crate::helpers::{get_random_email, TestApp};
use auth_service::routes::SignupResponse;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!(
            {
                "email": random_email,
                "password": "pwd"
            }
        ),
        serde_json::json!(
            {
                "email": random_email,
                "password": "pwd",
                "requires2FA": "true"
            }
        ),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let body = serde_json::json!(
        {
            "email": "test_user@email.com",
            "password": "12341234",
            "requires2FA": true
        }
    );
    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}
