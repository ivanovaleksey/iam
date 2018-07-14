use abac::types::AbacAttribute;
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;

pub use self::all::All;
pub use self::has_target::HasTarget;

mod all {
    use super::*;

    #[derive(Debug)]
    pub struct All(pub Vec<AbacAttribute>);

    impl Message for All {
        type Result = QueryResult<Vec<AbacAttribute>>;
    }

    impl Handler<All> for DbExecutor {
        type Result = QueryResult<Vec<AbacAttribute>>;

        fn handle(&mut self, msg: All, _ctx: &mut Self::Context) -> Self::Result {
            let conn = &self.0.get().expect("Failed to get a connection from pool");
            query(conn, &msg.0)
        }
    }

    fn query(conn: &PgConnection, attrs: &[AbacAttribute]) -> QueryResult<Vec<AbacAttribute>> {
        use abac::functions::abac_object_target;

        diesel::select(abac_object_target(attrs)).get_results(conn)
    }
}

mod has_target {
    use super::*;
    use abac::types::AbacAttributeSqlType;
    use diesel::sql_types::{Array, Bool};

    #[derive(Debug)]
    pub struct HasTarget(pub Vec<AbacAttribute>, pub AbacAttribute);

    impl Message for HasTarget {
        type Result = QueryResult<bool>;
    }

    impl Handler<HasTarget> for DbExecutor {
        type Result = QueryResult<bool>;

        fn handle(&mut self, msg: HasTarget, _ctx: &mut Self::Context) -> Self::Result {
            let conn = &self.0.get().expect("Failed to get a connection from pool");
            query(conn, &msg.0, &msg.1)
        }
    }

    #[derive(Debug, QueryableByName)]
    struct Exists(
        #[column_name = "exists"]
        #[sql_type = "Bool"]
        bool,
    );

    fn query(
        conn: &PgConnection,
        attrs: &[AbacAttribute],
        attr: &AbacAttribute,
    ) -> QueryResult<bool> {
        let query = r#"
            select exists(
                select 1
                from abac_object_target($1)
                where abac_object_target = $2
                limit 1
            )
        "#;

        diesel::dsl::sql_query(query)
            .bind::<Array<AbacAttributeSqlType>, _>(attrs)
            .bind::<AbacAttributeSqlType, _>(attr)
            .get_result::<Exists>(conn)
            .map(|res| res.0)
    }
}
