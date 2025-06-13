use crate::database::connection::AppState;
use crate::database::queries::{
    VeinWithStatus, insert_vein, insert_vein_confirmation, insert_vein_depletion,
    insert_vein_is_bedrock, search_veins,
};
use crate::models::forms::{AddVeinForm, SearchQuery};
use axum::{
    extract::{Form, Query, State},
    http::StatusCode,
    response::Html,
};
use uuid::Uuid;

pub async fn search_veins_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Html<String>, StatusCode> {
    let mut connection = match state.diesel_pool.get().await {
        Ok(conn) => conn,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match search_veins(&mut connection, &params).await {
        Ok(veins) => Ok(generate_search_results_html(veins, &params)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(Html(generate_database_error_html()))
        }
    }
}

pub async fn add_vein_handler(
    State(state): State<AppState>,
    Form(form): Form<AddVeinForm>,
) -> Result<Html<String>, StatusCode> {
    let mut connection = match state.diesel_pool.get().await {
        Ok(conn) => conn,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let id = Uuid::new_v4().to_string();

    // 座標の解析
    let x_coord = match form.parse_x_coord() {
        Ok(val) => val,
        Err(_) => return Ok(Html(generate_coord_error_html("X"))),
    };
    let y_coord = match form.parse_y_coord() {
        Ok(val) => val,
        Err(_) => return Ok(Html(generate_coord_error_html("Y"))),
    };
    let z_coord = match form.parse_z_coord() {
        Ok(val) => val,
        Err(_) => return Ok(Html(generate_coord_error_html("Z"))),
    };

    // 鉱脈の挿入
    if let Err(e) = insert_vein(
        &mut connection,
        &id,
        &form.name,
        x_coord,
        y_coord,
        z_coord,
        &form.notes,
    )
    .await
    {
        eprintln!("Database error: {}", e);
        return Ok(Html(generate_database_error_html()));
    }

    // 確認済みの場合
    if form.is_confirmed() {
        if let Err(e) = insert_vein_confirmation(&mut connection, &id, true).await {
            eprintln!("Failed to insert confirmation: {}", e);
        }
    }

    // 枯渇済みの場合
    if form.is_depleted() {
        if let Err(e) = insert_vein_depletion(&mut connection, &id, true).await {
            eprintln!("Failed to insert depletion: {}", e);
        }
    }

    // 岩盤鉱脈の場合
    if form.is_bedrock() {
        if let Err(e) = insert_vein_is_bedrock(&mut connection, &id, true).await {
            eprintln!("Failed to insert bedrock status: {}", e);
        }
    }

    Ok(Html(generate_success_html(&form, &id)))
}

fn generate_search_results_html(veins: Vec<VeinWithStatus>, query: &SearchQuery) -> Html<String> {
    let mut search_info = if query.has_name_filter() {
        format!("検索条件: 名前: {}", query.name.as_ref().unwrap())
    } else {
        "全ての鉱脈".to_string()
    };

    if query.should_include_revoked() {
        search_info.push_str(" (取り下げられた鉱脈を含む)");
    }

    let results_html = if veins.is_empty() {
        "<p>検索条件に一致する鉱脈が見つかりませんでした。</p>".to_string()
    } else {
        generate_veins_table(veins, query)
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="ja">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>検索結果 - GT6 鉱脈マネージャー</title>
            <link rel="stylesheet" href="/styles.css">
        </head>
        <body>
            <div class="container">
                <h1>検索結果</h1>
                <h2>{}</h2>
                {}
                
                <div class="nav-links">
                    <a href="/">戻る</a>
                </div>
            </div>
        </body>
        </html>
        "#,
        search_info, results_html
    ))
}

fn generate_veins_table(veins: Vec<VeinWithStatus>, query: &SearchQuery) -> String {
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
                <th>岩盤鉱脈</th>
                <th>視認済み</th>
                <th>枯渇済み</th>
                <th>登録日時</th>
                <th>操作</th>
            </tr>
        </thead>
        <tbody>
        "#,
    );

    for vein in veins {
        let row_class = if vein.revoked { "revoked-vein" } else { "" };

        let button_builder = |vein_id: &str,
                              target_state: &str,
                              target_operation: &str,
                              button_state: &str,
                              button_text: &str,
                              confirm_msg: Option<&str>| {
            let confirm_attr = match confirm_msg {
                Some(msg) => format!("onclick=\"return confirm('{}')\"", msg),
                None => "".to_string(),
            };
            format!(
                r#"
                <form style="display: inline;" method="post" action="/api/veins/{}/{}/{}">
                    <input type="hidden" name="query_state" value="{}">
                    <button type="submit" class="action-btn {}" {}>{}</button>
                </form>
                "#,
                vein_id,
                target_state,
                target_operation,
                query.get_all_query_string(),
                button_state,
                confirm_attr,
                button_text,
            )
        };

        let confirmation_button = if vein.revoked {
            "".to_string()
        } else if vein.confirmed {
            button_builder(
                &vein.id,
                "confirmation",
                "revoke",
                "confirmed",
                "視認解除",
                Some("この鉱脈の視認済みマークを解除しますか？"),
            )
        } else {
            button_builder(
                &vein.id,
                "confirmation",
                "set",
                "confirm",
                "視認済みにする",
                None,
            )
        };

        let depletion_button = if vein.revoked {
            "".to_string()
        } else if vein.depleted {
            button_builder(
                &vein.id,
                "depletion",
                "revoke",
                "depleted",
                "枯渇解除",
                Some("この鉱脈の枯渇マークを解除しますか？"),
            )
        } else {
            button_builder(
                &vein.id,
                "depletion",
                "set",
                "deplete",
                "枯渇済みにする",
                None,
            )
        };

        let revocation_button = if vein.revoked {
            button_builder(
                &vein.id,
                "revocation",
                "revoke",
                "revoked",
                "復元",
                Some("この鉱脈の登録を復元しますか？"),
            )
        } else {
            button_builder(
                &vein.id,
                "revocation",
                "set",
                "revoke",
                "取り下げ",
                Some("この鉱脈を取り下げますか？"),
            )
        };

        html.push_str(&format!(
            r#"
            <tr class="{}">
                <td><strong>{}</strong></td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td class="action-buttons">
                    {}
                    {}
                    {}
                </td>
            </tr>
            "#,
            row_class,
            vein.name,
            vein.x_coord,
            vein.z_coord,
            vein.format_y_coord(),
            vein.format_notes(),
            vein.is_bedrock_symbol(),
            vein.confirmed_symbol(),
            vein.depleted_symbol(),
            vein.format_created_at(),
            confirmation_button,
            depletion_button,
            revocation_button,
        ));
    }

    html.push_str("</tbody></table>");
    html
}

pub async fn issue_invitation_html() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html lang="ja">
        <head>
            <meta charset="UTF-8">
            <title>招待リンク発行</title>
            <link rel="stylesheet" href="/styles.css">
        </head>
        <body>
            <div class="container">
                <h1>招待リンク発行</h1>
                <p>招待リンクを発行するには、以下のボタンをクリックしてください。</p>
                <form method="post" action="/auth/issue-invitation">
                    <input type="email" name="email" placeholder="あなたのメールアドレス">
                    <button type="submit">招待リンクを発行</button>
                </form>
                <div class="nav-links">
                    <a href="/">戻る</a>
                </div>
            </div>
        </body>
        </html>
        "#,
    )
}

fn generate_coord_error_html(coord_name: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html lang="ja">
        <head>
            <meta charset="UTF-8">
            <title>エラー</title>
            <link rel="stylesheet" href="/styles.css">
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
    )
}

fn generate_database_error_html() -> String {
    r#"
    <!DOCTYPE html>
    <html lang="ja">
    <head>
        <meta charset="UTF-8">
        <title>エラー</title>
        <link rel="stylesheet" href="/styles.css">
    </head>
    <body class="error-page">
        <h1>データベースエラー</h1>
        <div class="error">
            鉱脈の処理中にエラーが発生しました。<br>
            同じ名前や座標の鉱脈が既に存在している可能性があります。
        </div>
        <a href="/">戻る</a>
    </body>
    </html>
    "#
    .to_string()
}

fn generate_success_html(form: &AddVeinForm, id: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html lang="ja">
        <head>
            <meta charset="UTF-8">
            <title>追加完了</title>
            <link rel="stylesheet" href="/styles.css">
        </head>
        <body class="result-page">
            <h1>鉱脈追加完了</h1>
            <div class="success">
                <strong>「{}」</strong> が正常に追加されました！<br>
                座標: X={}, Z={}, Y={}<br>
                ID: {}
            </div>
            <div class="nav-links">
                <a href="/">戻る</a>
            </div>
        </body>
        </html>
        "#,
        form.name, form.x_coord, form.z_coord, form.y_coord, id
    )
}
