create table abac_subject_attr (
  namespace_id uuid not null,
  subject_id uuid not null,
  key text,
  value text not null,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, subject_id, key, value)
);
