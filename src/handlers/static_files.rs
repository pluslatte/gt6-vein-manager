use axum::{http::StatusCode, response::Html};

pub async fn serve_index() -> Html<String> {
    match tokio::fs::read_to_string("/home/latte/gt6-vein-manager/public/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html(generate_error_html("index.html が見つかりませんでした。")),
    }
}

pub async fn serve_css() -> (StatusCode, [(&'static str, &'static str); 1], String) {
    match tokio::fs::read_to_string("/home/latte/gt6-vein-manager/public/styles.css").await {
        Ok(content) => (StatusCode::OK, [("content-type", "text/css")], content),
        Err(_) => (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain")],
            "CSS file not found".to_string(),
        ),
    }
}

fn generate_error_html(message: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html lang="ja">
        <head>
            <meta charset="UTF-8">
            <title>エラー</title>
            <link rel="stylesheet" href="styles.css">
        </head>
        <body class="error-page">
            <h1>エラー</h1>
            <div class="error">
                <p>{}</p>
            </div>
        </body>
        </html>
        "#,
        message
    )
}
