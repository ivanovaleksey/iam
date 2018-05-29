table! {
    abac_action_attr (namespace_id, action_id, key, value) {
        namespace_id -> Uuid,
        action_id -> Text,
        key -> Text,
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
    abac_policy (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value) {
        namespace_id -> Uuid,
        subject_namespace_id -> Uuid,
        subject_key -> Text,
        subject_value -> Text,
        object_namespace_id -> Uuid,
        object_key -> Text,
        object_value -> Text,
        action_namespace_id -> Uuid,
        action_key -> Text,
        action_value -> Text,
        created_at -> Timestamp,
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

joinable!(abac_action_attr -> namespace (namespace_id));
joinable!(abac_object_attr -> namespace (namespace_id));
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
