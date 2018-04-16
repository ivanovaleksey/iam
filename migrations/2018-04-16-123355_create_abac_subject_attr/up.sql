create table abac_subject_attr (
  namespace_id uuid not null,
  subject_id uuid not null,
  value text not null,

  foreign key (namespace_id) references namespace (id) on delete cascade,
  primary key (namespace_id, value, subject_id)
);
