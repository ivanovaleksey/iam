create table account(
  id uuid default uuid_generate_v4(),
  enabled boolean not null default false,
  constraints jsonb not null default '{}',
  deleted_at timestamptz,

  primary key (id)
);
