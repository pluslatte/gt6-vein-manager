// @generated automatically by Diesel CLI.

diesel::table! {
    invitation (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 36]
        token -> Char,
        #[max_length = 36]
        invited_by -> Nullable<Char>,
        expires_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        #[max_length = 36]
        used_by -> Nullable<Char>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sessions (id) {
        #[max_length = 255]
        id -> Varchar,
        data -> Longtext,
        expiry_date -> Timestamp,
    }
}

diesel::table! {
    user (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 255]
        password_hash -> Varchar,
        is_admin -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        #[max_length = 36]
        invited_by -> Nullable<Char>,
    }
}

diesel::table! {
    vein (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        x_coord -> Integer,
        y_coord -> Nullable<Integer>,
        z_coord -> Integer,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    vein_confirmation (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        vein_id -> Varchar,
        confirmed -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    vein_depletion (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        vein_id -> Varchar,
        depleted -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    vein_is_bedrock (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        vein_id -> Varchar,
        is_bedrock -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    vein_note (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        vein_id -> Varchar,
        #[max_length = 255]
        note -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    vein_revocation (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        vein_id -> Varchar,
        revoked -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(vein_confirmation -> vein (vein_id));
diesel::joinable!(vein_depletion -> vein (vein_id));
diesel::joinable!(vein_is_bedrock -> vein (vein_id));
diesel::joinable!(vein_note -> vein (vein_id));
diesel::joinable!(vein_revocation -> vein (vein_id));

diesel::allow_tables_to_appear_in_same_query!(
    invitation,
    sessions,
    user,
    vein,
    vein_confirmation,
    vein_depletion,
    vein_is_bedrock,
    vein_note,
    vein_revocation,
);
