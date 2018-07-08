table! {
    account (id) {
        id -> Uuid,
        constraints -> Jsonb,
        disabled_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

table! {
    identity (provider, label, uid) {
        provider -> Uuid,
        label -> Text,
        uid -> Text,
        account_id -> Uuid,
        created_at -> Timestamptz,
    }
}

table! {
    namespace (id) {
        id -> Uuid,
        label -> Text,
        account_id -> Uuid,
        deleted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

table! {
    refresh_token (account_id) {
        account_id -> Uuid,
        algorithm -> Text,
        keys -> Array<Bytea>,
        created_at -> Timestamptz,
    }
}

joinable!(identity -> account (account_id));
joinable!(identity -> namespace (provider));
joinable!(namespace -> account (account_id));
joinable!(refresh_token -> account (account_id));

allow_tables_to_appear_in_same_query!(account, identity, namespace, refresh_token,);
