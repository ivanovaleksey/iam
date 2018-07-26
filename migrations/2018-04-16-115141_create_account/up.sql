create table account(
  id uuid default gen_random_uuid(),
  constraints jsonb not null default '{}',
  disabled_at timestamptz,
  deleted_at timestamptz,

  primary key (id)
);
