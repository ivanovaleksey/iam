create table abac_action_attr (
  namespace_id uuid not null,
  action_id text not null,
  value text not null,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, value, action_id)
);
