mod abac_subject_attr;
mod identity;
mod namespace;

pub mod helpers {
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use iam;
    use iam::models;
    use iam::schema::*;
    use uuid::Uuid;

    pub fn can(
        conn: &PgConnection,
        namespace_ids: Vec<Uuid>,
        subject_id: Uuid,
        object_id: &str,
        action_id: &str,
    ) -> bool {
        let msg = iam::actors::db::auth::Auth {
            namespace_ids,
            subject: subject_id,
            object: object_id.to_owned(),
            action: action_id.to_owned(),
        };
        iam::rpc::auth::call(conn, &msg).unwrap()
    }

    pub fn iam_namespace(conn: &PgConnection) -> models::Namespace {
        namespace::table
            .filter(namespace::label.eq("iam.ng.services"))
            .first::<models::Namespace>(conn)
            .expect("Can't find IAM namespace")
    }
}

// Use case:
//     Пользователь фоксфорда "создает себя" через endpoint аутентификации.
//     Потом этому пользователю назначается роль `client`. Пользователь должен получить все права.
