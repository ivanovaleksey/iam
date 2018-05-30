use actix::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacObjectAttr;
use rpc::abac_object_attr::{create, delete, list, read};
use rpc::error::Result;

#[derive(Debug)]
pub struct Create {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Create {
    type Result = Result<AbacObjectAttr>;
}

impl Handler<Create> for DbExecutor {
    type Result = Result<AbacObjectAttr>;

    fn handle(&mut self, msg: Create, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        create::call(conn, msg)
    }
}

impl From<create::Request> for Create {
    fn from(req: create::Request) -> Self {
        Create {
            namespace_id: req.namespace_id,
            object_id: req.object_id,
            key: req.key,
            value: req.value,
        }
    }
}

#[derive(Debug)]
pub struct Read {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Read {
    type Result = Result<AbacObjectAttr>;
}

impl Handler<Read> for DbExecutor {
    type Result = Result<AbacObjectAttr>;

    fn handle(&mut self, msg: Read, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        read::call(conn, msg)
    }
}

impl From<read::Request> for Read {
    fn from(req: read::Request) -> Self {
        Read {
            namespace_id: req.namespace_id,
            object_id: req.object_id,
            key: req.key,
            value: req.value,
        }
    }
}

#[derive(Debug)]
pub struct Delete {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Delete {
    type Result = Result<AbacObjectAttr>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<AbacObjectAttr>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        delete::call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            namespace_id: req.namespace_id,
            object_id: req.object_id,
            key: req.key,
            value: req.value,
        }
    }
}

#[derive(Debug)]
pub struct List {
    pub namespace_id: Uuid,
    pub object_id: Option<String>,
    pub key: Option<String>,
}

impl Message for List {
    type Result = Result<Vec<AbacObjectAttr>>;
}

impl Handler<List> for DbExecutor {
    type Result = Result<Vec<AbacObjectAttr>>;

    fn handle(&mut self, msg: List, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        list::call(conn, msg)
    }
}

impl From<list::Request> for List {
    fn from(req: list::Request) -> Self {
        let filter = req.filter.0;
        List {
            namespace_id: filter.namespace_id,
            object_id: filter.object_id,
            key: filter.key,
        }
    }
}
