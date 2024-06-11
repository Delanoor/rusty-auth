use crate::helpers::{get_random_email, TestApp};
use auth_service::{routes::SignupResponse, ErrorResponse};

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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let body = serde_json::json!(
        {
            "email": "testuser@gmail.com",
            "password": "pwds1234",
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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters

    // Create an array of invalid inputs. Then, iterate through the array and
    // make HTTP calls to the signup route. Assert a 400 HTTP status code is returned.
    let input: [serde_json::Value; 2] = [
        serde_json::json!(
            {
                "email": "abc",
                "password": "pwds1234",
                "requires2FA": false,
            }
        ),
        serde_json::json!(
            {
                "email": "test_email@email.com",
                "password": "pwd",
                "requires2FA": false,
            }
        ),
    ];

    for i in input.iter() {
        let response = app.post_signup(i).await;
        assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", i);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "test_409@email.com",
        "password": "pwds1234",
        "requires2FA": false,
    });

    let response = app.post_signup(&body).await;
    assert_eq!(response.status().as_u16(), 201);

    let response_second = app.post_signup(&body).await;
    assert_eq!(response_second.status().as_u16(), 409);

    assert_eq!(
        response_second
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );

    app.clean_up().await;
}
