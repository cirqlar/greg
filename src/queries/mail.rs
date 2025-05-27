use std::env;

pub async fn send_email(
    subject: &str,
    text: &str,
    html: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();

    client
        .post(env::var("MAIL_URL").expect("MAIL_URL should be set"))
        .bearer_auth(env::var("MAIL_TOKEN").expect("MAIL_TOKEN should be set"))
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "from": {
                    "email": env::var("FROM_EMAIL").expect("FROM_EMAIL should be set"),
                    "name": env::var("FROM_NAME").expect("FROM_NAME should be set")
                },
                "to": [{
                    "email": env::var("TO_EMAIL").expect("TO_EMAIL should be set"),
                    "name": env::var("TO_NAME").expect("TO_NAME should be set")
                }],
                "subject": subject,
                "text": text,
                "html": html
            })
            .to_string(),
        )
        .send()
        .await
}
