create table namespace (
  id uuid default gen_random_uuid(),
  label text not null,
  account_id uuid not null,
  deleted_at timestamptz,
  created_at timestamptz not null default now(),

  unique(label),
  foreign key (account_id) references account (id) on delete cascade,
  primary key (id)
);
