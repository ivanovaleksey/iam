use abac::Attribute;
use uuid::Uuid;

use std::fmt;

use models::identity;

#[derive(Clone, Debug)]
pub enum UriKind {
    Account(Uuid),
    Identity(identity::PrimaryKey),
    Namespace(Uuid),
}

impl Attribute for UriKind {
    fn key(&self) -> String {
        "uri".to_owned()
    }

    fn value(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for UriKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UriKind::*;

        match self {
            Account(id) => write!(f, "account/{}", id),
            Identity(id) => write!(f, "identity/{}", id),
            Namespace(id) => write!(f, "namespace/{}", id),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CollectionKind {
    Account,
    Identity,
    Namespace,
    AbacAction,
    AbacObject,
    AbacSubject,
    AbacPolicy,
}

impl Attribute for CollectionKind {
    fn key(&self) -> String {
        "type".to_owned()
    }

    fn value(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for CollectionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CollectionKind::*;

        let v = match self {
            Account => "account",
            Identity => "identity",
            Namespace => "namespace",
            AbacAction => "abac_action",
            AbacObject => "abac_object",
            AbacSubject => "abac_subject",
            AbacPolicy => "abac_policy",
        };
        write!(f, "{}", v)
    }
}

#[derive(Clone, Debug)]
pub enum OperationKind {
    Create,
    Read,
    Update,
    Delete,
    List,
    Any,
}

impl Attribute for OperationKind {
    fn key(&self) -> String {
        "operation".to_owned()
    }

    fn value(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for OperationKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OperationKind::*;

        let v = match self {
            Create => "create",
            Read => "read",
            Update => "update",
            Delete => "delete",
            List => "list",
            Any => "any",
        };
        write!(f, "{}", v)
    }
}
