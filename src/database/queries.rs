use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::{Vein, SearchQuery};
use anyhow::Result;

const BASE_QUERY: &str = r#"
    SELECT
        v.id, v.name, v.x_coord, v.y_coord, v.z_coord, v.notes, v.created_at,
        CASE WHEN vc.confirmed IS NOT NULL THEN vc.confirmed ELSE FALSE END AS confirmed,
        CASE WHEN vd.depleted IS NOT NULL THEN vd.depleted ELSE FALSE END AS depleted,
        CASE WHEN vr.revoked IS NOT NULL THEN vr.revoked ELSE FALSE END AS revoked
    FROM veins v
        LEFT JOIN (
            SELECT DISTINCT vein_id, confirmed
            FROM vein_confirmations vc1
            WHERE vc1.created_at = (
                SELECT MAX(vc2.created_at)
                FROM vein_confirmations vc2
                WHERE vc2.vein_id = vc1.vein_id
            )
        ) vc ON v.id = vc.vein_id
        LEFT JOIN (
            SELECT DISTINCT vein_id, depleted
            FROM vein_depletions vd1
            WHERE vd1.created_at = (
                SELECT MAX(vd2.created_at)
                FROM vein_depletions vd2
                WHERE vd2.vein_id = vd1.vein_id
            )
        ) vd ON v.id = vd.vein_id
        LEFT JOIN (
            SELECT DISTINCT vein_id, revoked
            FROM vein_revokations vr1
            WHERE vr1.created_at = (
                SELECT MAX(vr2.created_at)
                FROM vein_revokations vr2
                WHERE vr2.vein_id = vr1.vein_id
            )
        ) vr ON v.id = vr.vein_id
    WHERE
        (vr.revoked IS NULL OR vr.revoked = FALSE)
"#;

pub async fn get_all_veins(pool: &MySqlPool) -> Result<Vec<Vein>, sqlx::Error> {
    let query = format!("{} ORDER BY v.created_at DESC", BASE_QUERY);
    
    sqlx::query_as::<_, Vein>(&query)
        .fetch_all(pool)
        .await
}

pub async fn search_veins(pool: &MySqlPool, search_query: &SearchQuery) -> Result<Vec<Vein>, sqlx::Error> {
    let mut query = BASE_QUERY.to_string();
    let mut conditions = Vec::new();

    if let Some(name) = search_query.get_name_filter() {
        conditions.push(format!("name LIKE '%{}%'", name.replace("'", "''")));
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY created_at DESC");

    sqlx::query_as::<_, Vein>(&query)
        .fetch_all(pool)
        .await
}

pub async fn insert_vein(
    pool: &MySqlPool,
    id: &str,
    name: &str,
    x_coord: i32,
    y_coord: Option<i32>,
    z_coord: i32,
    notes: &Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO veins (id, name, x_coord, y_coord, z_coord, notes)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(x_coord)
    .bind(y_coord)
    .bind(z_coord)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_vein_confirmation(
    pool: &MySqlPool,
    vein_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO vein_confirmations (id, vein_id, confirmed)
        VALUES (?, ?, TRUE)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(vein_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_vein_depletion(
    pool: &MySqlPool,
    vein_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO vein_depletions (id, vein_id, depleted)
        VALUES (?, ?, TRUE)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(vein_id)
    .execute(pool)
    .await?;

    Ok(())
}
