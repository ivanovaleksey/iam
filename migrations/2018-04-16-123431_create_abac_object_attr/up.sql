create table abac_object_attr (
  namespace_id uuid,
  object_id text,
  key text,
  value text,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, object_id, key, value)
);
