use axum::{
    Json, Router,
    extract::{Form, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db_pool: sqlx::Pool<sqlx::MySql>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Vein {
    id: String,
    name: String,
    x_coord: i32,
    y_coord: Option<i32>,
    z_coord: i32,
    notes: Option<String>,
    confirmed: bool,
    depleted: bool,
    created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    name: Option<String>,
    x_coord: Option<i32>,
    y_coord: Option<i32>,
    z_coord: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct AddVeinForm {
    name: String,
    x_coord: String,
    y_coord: String,
    z_coord: String,
    notes: Option<String>,
    confirmed: Option<bool>,
    depleted: Option<bool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.");

    let pool = MySqlPool::connect(&database_url).await?;
    println!("Connected to the database at {}", database_url);

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS veins (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            x_coord INT NOT NULL,
            y_coord INT DEFAULT NULL,
            z_coord INT NOT NULL,
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vein_confirmations (
            id VARCHAR(36) PRIMARY KEY,
            vein_id VARCHAR(36) NOT NULL,
            confirmed BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vein_id) REFERENCES veins(id) ON DELETE CASCADE
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vein_depletions (
            id VARCHAR(36) PRIMARY KEY,
            vein_id VARCHAR(36) NOT NULL,
            depleted BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vein_id) REFERENCES veins(id) ON DELETE CASCADE
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vein_revokations (
            id VARCHAR(36) PRIMARY KEY,
            vein_id VARCHAR(36) NOT NULL,
            revoked BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vein_id) REFERENCES veins(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/api/veins", get(get_veins_all))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/styles.css", get(serve_css))
        .route("/search", get(search_veins))
        .route("/add", post(add_vein))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:24528").await?;
    println!("Server running on http://localhost:24528");

    axum::serve(listener, app).await?;

    anyhow::Ok(())
}

async fn get_veins_all(State(state): State<AppState>) -> Result<Json<Vec<Vein>>, StatusCode> {
    let veins = sqlx::query_as::<_, Vein>(
        r#"
        SELECT id, name, x_coord, y_coord, z_coord, notes, confirmed, depleted, created_at
        FROM veins
        ORDER BY created_at DESC
    "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(veins))
}

async fn serve_index() -> Html<String> {
    match tokio::fs::read_to_string("/home/latte/gt6-vein-manager/public/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html(
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
                    <p>index.html が見つかりませんでした。</p>
                </div>
            </body>
            </html>
            "#
            .to_string(),
        ),
    }
}

async fn serve_css() -> (StatusCode, [(&'static str, &'static str); 1], String) {
    match tokio::fs::read_to_string("/home/latte/gt6-vein-manager/public/styles.css").await {
        Ok(content) => (StatusCode::OK, [("content-type", "text/css")], content),
        Err(_) => (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain")],
            "CSS file not found".to_string(),
        ),
    }
}

async fn search_veins(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Html<String> {
    let mut query =
        "SELECT id, name, x_coord, y_coord, z_coord, notes, confirmed, depleted, created_at FROM veins WHERE 1=1"
            .to_string();
    let mut conditions = Vec::new();

    if let Some(name) = &params.name {
        if !name.trim().is_empty() {
            conditions.push(format!("name LIKE '%{}%'", name.replace("'", "''")));
        }
    }

    if let Some(x) = params.x_coord {
        conditions.push(format!("x_coord = {}", x));
    }

    if let Some(z) = params.z_coord {
        conditions.push(format!("z_coord = {}", z));
    }

    if let Some(y) = params.y_coord {
        conditions.push(format!("y_coord = {}", y));
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY created_at DESC");

    match sqlx::query_as::<_, Vein>(&query)
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(veins) => generate_search_results_html(veins, &params),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Html(
                r#"
                <!DOCTYPE html>
                <html lang="ja">
                <head>
                    <meta charset="UTF-8">
                    <title>エラー</title>
                    <link rel="stylesheet" href="styles.css">
                </head>
                <body class="error-page">
                    <h1>検索エラー</h1>
                    <div class="error">
                        <p>データベースエラーが発生しました。</p>
                    </div>
                    <a href="/">戻る</a>
                </body>
                </html>
                "#
                .to_string(),
            )
        }
    }
}

async fn add_vein(State(state): State<AppState>, Form(form): Form<AddVeinForm>) -> Html<String> {
    let id = Uuid::new_v4().to_string();

    // Helper function for error HTML
    fn coord_error_html(coord_name: &str) -> Html<String> {
        Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
                <meta charset="UTF-8">
                <title>エラー</title>
                <link rel="stylesheet" href="styles.css">
            </head>
            <body class="error-page">
                <h1>追加エラー</h1>
                <div class="error">
                    {}座標が正しい整数ではありません。<br>
                </div>
                <a href="/">戻る</a>
            </body>
            </html>
            "#,
            coord_name
        ))
    }

    // Validate and parse coordinates
    let x_coord = match form.x_coord.parse::<i32>() {
        Ok(val) => val,
        Err(_) => return coord_error_html("X"),
    };
    let y_coord = if form.y_coord.trim().is_empty() {
        None
    } else {
        match form.y_coord.parse::<i32>() {
            Ok(val) => Some(val),
            Err(_) => return coord_error_html("Y"),
        }
    };
    let z_coord = match form.z_coord.parse::<i32>() {
        Ok(val) => val,
        Err(_) => return coord_error_html("Z"),
    };

    let result = sqlx::query(
        r#"
        INSERT INTO veins (id, name, x_coord, y_coord, z_coord, notes, confirmed, depleted)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&form.name)
    .bind(x_coord)
    .bind(y_coord)
    .bind(z_coord)
    .bind(&form.notes)
    .bind(form.confirmed.unwrap_or(false))
    .bind(form.depleted.unwrap_or(false))
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
                <meta charset="UTF-8">
                <title>追加完了</title>
                <link rel="stylesheet" href="styles.css">
            </head>
            <body class="result-page">
                <h1>鉱脈追加完了</h1>
                <div class="success">
                    <strong>「{}」</strong> が正常に追加されました！<br>
                    座標: X={}, Z={}, Y={}<br>
                    ID: {}
                </div>
                <a href="/">戻る</a> | <a href="/search">全ての鉱脈を表示</a>
            </body>
            </html>
            "#,
            &form.name, &form.x_coord, &form.z_coord, &form.y_coord, id
        )),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Html(
                r#"
                <!DOCTYPE html>
                <html lang="ja">
                <head>
                    <meta charset="UTF-8">
                    <title>エラー</title>
                    <link rel="stylesheet" href="styles.css">
                </head>
                <body class="error-page">
                    <h1>データベースエラー</h1>
                    <div class="error">
                        鉱脈の追加中にエラーが発生しました。<br>
                        同じ名前や座標の鉱脈が既に存在している可能性があります。
                    </div>
                    <a href="/">戻る</a>
                </body>
                </html>
                "#
                .to_string(),
            )
        }
    }
}

fn generate_search_results_html(veins: Vec<Vein>, query: &SearchQuery) -> Html<String> {
    let search_info = {
        let mut info_parts = Vec::new();
        if let Some(name) = &query.name {
            if !name.trim().is_empty() {
                info_parts.push(format!("名前: {}", name));
            }
        }
        if let Some(x) = query.x_coord {
            info_parts.push(format!("X座標: {}", x));
        }
        if let Some(z) = query.z_coord {
            info_parts.push(format!("Z座標: {}", z));
        }
        if let Some(y) = query.y_coord {
            info_parts.push(format!("Y座標: {}", y));
        }

        if info_parts.is_empty() {
            "全ての鉱脈".to_string()
        } else {
            format!("検索条件: {}", info_parts.join(", "))
        }
    };

    let results_html = if veins.is_empty() {
        "<p>検索条件に一致する鉱脈が見つかりませんでした。</p>".to_string()
    } else {
        let mut html = format!("<p>{} 件の鉱脈が見つかりました。</p>", veins.len());
        html.push_str("<table>");
        html.push_str(
            r#"
            <thead>
                <tr>
                    <th>名前</th>
                    <th>X座標</th>
                    <th>Z座標</th>
                    <th>Y座標</th>
                    <th>メモ</th>
                    <th>視認済み</th>
                    <th>枯渇済み</th>
                    <th>登録日時</th>
                </tr>
            </thead>
            <tbody>
        "#,
        );

        for vein in veins {
            html.push_str(&format!(
                r#"
                <tr>
                    <td><strong>{}</strong></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                "#,
                vein.name,
                vein.x_coord,
                vein.z_coord,
                vein.y_coord
                    .map_or_else(|| "-".to_string(), |y| y.to_string()),
                vein.notes.as_deref().unwrap_or("-"),
                if vein.confirmed { "✓" } else { "✗" },
                if vein.depleted { "✓" } else { "✗" },
                vein.created_at.map_or_else(
                    || "-".to_string(),
                    |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()
                )
            ));
        }

        html.push_str("</tbody></table>");
        html
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="ja">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>検索結果 - GT6 鉱脈マネージャー</title>
            <link rel="stylesheet" href="styles.css">
        </head>
        <body>
            <div class="container">
                <h1>検索結果</h1>
                <h2>{}</h2>
                {}
                
                <div class="nav-links">
                    <a href="/">新しい検索</a>
                    <a href="/search">全ての鉱脈を表示</a>
                </div>
            </div>
        </body>
        </html>
        "#,
        search_info, results_html
    ))
}
