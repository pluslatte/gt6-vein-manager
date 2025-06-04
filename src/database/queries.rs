use crate::models::SearchQuery;
use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, QueryResult, SelectableHelper,
    TextExpressionMethods, insert_into,
};
use diesel_async::{AsyncMysqlConnection, RunQueryDsl};
use gt6_vein_manager::schema::*;
use uuid::Uuid;

pub struct VeinWithStatus {
    pub id: String,
    pub name: String,
    pub x_coord: i32,
    pub y_coord: Option<i32>,
    pub z_coord: i32,
    pub notes: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub confirmed: bool,
    pub depleted: bool,
    pub revoked: bool,
    pub is_bedrock: bool,
}

impl VeinWithStatus {
    pub fn format_y_coord(&self) -> String {
        self.y_coord
            .map_or_else(|| "-".to_string(), |y| y.to_string())
    }

    pub fn format_notes(&self) -> &str {
        self.notes.as_deref().unwrap_or("-")
    }

    pub fn format_created_at(&self) -> String {
        self.created_at.map_or_else(
            || "-".to_string(),
            |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        )
    }

    pub fn confirmed_symbol(&self) -> &'static str {
        if self.confirmed {
            "はい"
        } else {
            "いいえ"
        }
    }

    pub fn depleted_symbol(&self) -> &'static str {
        if self.depleted { "はい" } else { "いいえ" }
    }
}

pub async fn search_veins(
    connection: &mut AsyncMysqlConnection,
    search_query: &SearchQuery,
) -> QueryResult<Vec<VeinWithStatus>> {
    use gt6_vein_manager::schema::vein::dsl::*;
    use gt6_vein_manager::schema::vein_confirmation::dsl as vc_dsl;
    use gt6_vein_manager::schema::vein_depletion::dsl as vd_dsl;
    use gt6_vein_manager::schema::vein_is_bedrock::dsl as vb_dsl;
    use gt6_vein_manager::schema::vein_note::dsl as vn_dsl;
    use gt6_vein_manager::schema::vein_revocation::dsl as vr_dsl;

    let mut query = vein.into_boxed();

    // Apply name filter if provided
    if let Some(name_filter) = search_query.get_name_filter() {
        query = query.filter(name.like(format!("%{}%", name_filter)));
    }

    // Execute the main query to get veins
    let veins: Vec<crate::models::Vein> = query
        .select(crate::models::Vein::as_select())
        .load(connection)
        .await?;

    let mut results = Vec::new();

    for vein_record in veins {
        // Get latest confirmation status
        let latest_confirmed = vein_confirmation::table
            .filter(vc_dsl::vein_id.eq(&vein_record.id))
            .order(vc_dsl::created_at.desc())
            .select(vc_dsl::confirmed)
            .first::<Option<bool>>(connection)
            .await
            .optional()?
            .flatten()
            .unwrap_or(false);

        // Get latest depletion status
        let latest_depleted = vein_depletion::table
            .filter(vd_dsl::vein_id.eq(&vein_record.id))
            .order(vd_dsl::created_at.desc())
            .select(vd_dsl::depleted)
            .first::<Option<bool>>(connection)
            .await
            .optional()?
            .flatten()
            .unwrap_or(false);

        // Get latest revocation status
        let latest_revoked = vein_revocation::table
            .filter(vr_dsl::vein_id.eq(&vein_record.id))
            .order(vr_dsl::created_at.desc())
            .select(vr_dsl::revoked)
            .first::<Option<bool>>(connection)
            .await
            .optional()?
            .flatten()
            .unwrap_or(false);

        // Get latest bedrock status
        let latest_is_bedrock = vein_is_bedrock::table
            .filter(vb_dsl::vein_id.eq(&vein_record.id))
            .order(vb_dsl::created_at.desc())
            .select(vb_dsl::is_bedrock)
            .first::<Option<bool>>(connection)
            .await
            .optional()?
            .flatten()
            .unwrap_or(false);

        // Get latest note
        let latest_notes = vein_note::table
            .filter(vn_dsl::vein_id.eq(&vein_record.id))
            .order(vn_dsl::created_at.desc())
            .select(vn_dsl::note)
            .first::<Option<String>>(connection)
            .await
            .optional()?
            .flatten();

        // Skip revoked veins unless explicitly requested
        if latest_revoked && !search_query.should_include_revoked() {
            continue;
        }

        let vein_with_status = VeinWithStatus {
            id: vein_record.id,
            name: vein_record.name,
            x_coord: vein_record.x_coord,
            y_coord: vein_record.y_coord,
            z_coord: vein_record.z_coord,
            notes: latest_notes,
            created_at: vein_record.created_at,
            confirmed: latest_confirmed,
            depleted: latest_depleted,
            revoked: latest_revoked,
            is_bedrock: latest_is_bedrock,
        };

        results.push(vein_with_status);
    }

    Ok(results)
}

pub async fn insert_vein(
    connection: &mut AsyncMysqlConnection,
    id: &str,
    name: &str,
    x_coord: i32,
    y_coord: Option<i32>,
    z_coord: i32,
    notes: &Option<String>,
) -> QueryResult<usize> {
    println!(
        "Attempting to insert vein: id={}, name={}, x_coord={}, y_coord={:?}, z_coord={}",
        id, name, x_coord, y_coord, z_coord
    );
    insert_into(vein::table)
        .values((
            vein::id.eq(id),
            vein::name.eq(name),
            vein::x_coord.eq(x_coord),
            vein::y_coord.eq(y_coord),
            vein::z_coord.eq(z_coord),
        ))
        .execute(connection)
        .await
}

pub async fn insert_vein_note(
    connection: &mut AsyncMysqlConnection,
    vein_id: &str,
    note: &str,
) -> QueryResult<usize> {
    println!(
        "Attempting to insert vein note: vein_id={}, note={}",
        vein_id, note
    );
    insert_into(vein_note::table)
        .values((
            vein_note::id.eq(Uuid::new_v4().to_string()),
            vein_note::vein_id.eq(vein_id),
            vein_note::note.eq(note),
        ))
        .execute(connection)
        .await
}

pub async fn insert_vein_confirmation(
    connection: &mut AsyncMysqlConnection,
    vein_id: &str,
    confirmed: bool,
) -> QueryResult<usize> {
    println!(
        "Attempting to insert vein confirmation: vein_id={}, confirmed={}",
        vein_id, confirmed
    );
    insert_into(vein_confirmation::table)
        .values((
            vein_confirmation::id.eq(Uuid::new_v4().to_string()),
            vein_confirmation::vein_id.eq(vein_id),
            vein_confirmation::confirmed.eq(confirmed),
        ))
        .execute(connection)
        .await
}

pub async fn insert_vein_depletion(
    connection: &mut AsyncMysqlConnection,
    vein_id: &str,
    depleted: bool,
) -> QueryResult<usize> {
    println!(
        "Attempting to insert vein depletion: vein_id={}, depleted={}",
        vein_id, depleted
    );
    insert_into(vein_depletion::table)
        .values((
            vein_depletion::id.eq(Uuid::new_v4().to_string()),
            vein_depletion::vein_id.eq(vein_id),
            vein_depletion::depleted.eq(depleted),
        ))
        .execute(connection)
        .await
}

pub async fn insert_vein_revocation(
    connection: &mut AsyncMysqlConnection,
    vein_id: &str,
    revoked: bool,
) -> QueryResult<usize> {
    println!(
        "Attempting to insert vein revocation: vein_id={}, revoked={}",
        vein_id, revoked
    );
    insert_into(vein_revocation::table)
        .values((
            vein_revocation::id.eq(Uuid::new_v4().to_string()),
            vein_revocation::vein_id.eq(vein_id),
            vein_revocation::revoked.eq(revoked),
        ))
        .execute(connection)
        .await
}
