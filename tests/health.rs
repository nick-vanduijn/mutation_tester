use reqwest::StatusCode;

#[tokio::test]
async fn health_check_works() {
    let resp = reqwest::get("http://localhost:3000/health").await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
