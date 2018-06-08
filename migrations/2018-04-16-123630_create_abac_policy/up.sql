create table abac_policy (
  namespace_id uuid not null,
  subject_namespace_id uuid not null,
  subject_key text not null,
  subject_value text not null,
  object_namespace_id uuid not null,
  object_key text not null,
  object_value text not null,
  action_namespace_id uuid not null,
  action_key text not null,
  action_value text not null,
  created_at timestamp not null default now(),
  not_before timestamp,
  expired_at timestamp,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  foreign key (subject_namespace_id) references namespace (id) on delete cascade,
  foreign key (object_namespace_id) references namespace (id) on delete cascade,
  foreign key (action_namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value)
);
