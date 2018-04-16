create table abac_policy (
  id uuid,
  namespace_id uuid not null,
  subject_value text not null,
  object_value text not null,
  action_value text not null,
  issued_at timestamp not null default now(),
  not_before timestamp,
  expired_at timestamp,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (id)
);
