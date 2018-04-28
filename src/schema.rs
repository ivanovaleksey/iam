table! {
    abac_action_attr (namespace_id, value, action_id) {
        namespace_id -> Uuid,
        action_id -> Text,
        value -> Text,
    }
}

table! {
    abac_object_attr (namespace_id, object_id, key, value) {
        namespace_id -> Uuid,
        object_id -> Text,
        key -> Text,
        value -> Text,
    }
}

table! {
    abac_policy (id) {
        id -> Uuid,
        namespace_id -> Uuid,
        subject_value -> Text,
        object_value -> Text,
        action_value -> Text,
        issued_at -> Timestamp,
        not_before -> Nullable<Timestamp>,
        expired_at -> Nullable<Timestamp>,
    }
}

table! {
    abac_subject_attr (namespace_id, subject_id, key, value) {
        namespace_id -> Uuid,
        subject_id -> Uuid,
        key -> Text,
        value -> Text,
    }
}

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
        issuer_id -> Uuid,
        account_id -> Uuid,
        issued_at -> Timestamp,
    }
}

table! {
    namespace (id) {
        id -> Uuid,
        label -> Text,
        account_id -> Uuid,
        enabled -> Bool,
        issued_at -> Timestamp,
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

joinable!(abac_action_attr -> namespace (namespace_id));
joinable!(abac_object_attr -> namespace (namespace_id));
joinable!(abac_policy -> namespace (namespace_id));
joinable!(abac_subject_attr -> namespace (namespace_id));
joinable!(identity -> namespace (provider));
joinable!(namespace -> account (account_id));
joinable!(refresh_token -> account (account_id));

allow_tables_to_appear_in_same_query!(
    abac_action_attr,
    abac_object_attr,
    abac_policy,
    abac_subject_attr,
    account,
    identity,
    namespace,
    refresh_token,
);
