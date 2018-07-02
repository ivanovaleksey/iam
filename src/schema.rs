table! {
    account (id) {
        id -> Uuid,
        enabled -> Bool,
        constraints -> Jsonb,
    }
}

table! {
    identity (provider, label, uid) {
        provider -> Uuid,
        label -> Text,
        uid -> Text,
        account_id -> Uuid,
        created_at -> Timestamp,
    }
}

table! {
    namespace (id) {
        id -> Uuid,
        label -> Text,
        account_id -> Uuid,
        enabled -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    refresh_token (account_id) {
        account_id -> Uuid,
        algorithm -> Text,
        keys -> Array<Bytea>,
        issued_at -> Timestamp,
    }
}

joinable!(identity -> namespace (provider));
joinable!(namespace -> account (account_id));
joinable!(refresh_token -> account (account_id));

allow_tables_to_appear_in_same_query!(account, identity, namespace, refresh_token,);
