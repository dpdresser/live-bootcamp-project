use crate::helpers::TestApp;

#[tokio::test]
async fn signup_succeeds() {
    let app = TestApp::new().await;

    let response = app.post_signup().await;

    assert_eq!(response.status().as_u16(), 200);
}