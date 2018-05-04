create table abac_action_attr (
  namespace_id uuid,
  action_id text,
  key text,
  value text,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, action_id, key, value)
);
