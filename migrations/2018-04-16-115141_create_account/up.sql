create table account(
  id uuid default uuid_generate_v4(),
  constraints jsonb not null default '{}',
  disabled_at timestamptz,
  deleted_at timestamptz,

  primary key (id)
);
