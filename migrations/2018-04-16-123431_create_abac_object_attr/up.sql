create table abac_object_attr (
  namespace_id uuid not null,
  object_id text not null,
  value text not null,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, value, object_id)
);
