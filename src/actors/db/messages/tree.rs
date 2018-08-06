use abac::AbacAttribute;
use actix::prelude::*;
use diesel::prelude::*;
use rpc::DirectionKind;

use actors::DbExecutor;

#[derive(Debug)]
pub enum CollectionKind {
    AbacSubject,
    AbacObject,
    AbacAction,
}

#[derive(Debug)]
pub struct Select {
    pub direction: DirectionKind,
    pub attribute: AbacAttribute,
    pub limit: u16,
    pub offset: u16,
}

#[derive(Debug)]
pub struct Tree(pub CollectionKind, pub Select);

impl Message for Tree {
    type Result = QueryResult<Vec<AbacAttribute>>;
}

impl Handler<Tree> for DbExecutor {
    type Result = QueryResult<Vec<AbacAttribute>>;

    fn handle(&mut self, msg: Tree, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, &msg)
    }
}

macro_rules! tree_query {
    ($table:ident, $msg:expr) => {{
        use abac::schema::$table;

        let msg = $msg;

        // Explicitly specifying non-default select clause is required to set desired
        // select type: (AbacAttribute) instead of (AbacAttribute, AbacAttribute, Timestamptz).
        // A correct column is then specified based on direction kind.
        let mut query = $table::table
            .select($table::inbound)
            .order($table::created_at.asc())
            .limit(i64::from(msg.limit))
            .offset(i64::from(msg.offset))
            .into_boxed();

        match msg.direction {
            DirectionKind::Inbound => {
                query = query
                    .select($table::inbound)
                    .filter($table::outbound.eq(&msg.attribute));
            }
            DirectionKind::Outbound => {
                query = query
                    .select($table::outbound)
                    .filter($table::inbound.eq(&msg.attribute));
            }
        }

        query
    }};
}

fn call(conn: &PgConnection, msg: &Tree) -> QueryResult<Vec<AbacAttribute>> {
    use self::CollectionKind::*;

    match msg.0 {
        AbacSubject => {
            let query = tree_query!(abac_subject, &msg.1);
            query.load(conn)
        }
        AbacObject => {
            let query = tree_query!(abac_object, &msg.1);
            query.load(conn)
        }
        AbacAction => {
            let query = tree_query!(abac_action, &msg.1);
            query.load(conn)
        }
    }
}
