create table namespace (
  id uuid default uuid_generate_v4(),
  label text not null,
  account_id uuid not null,
  deleted_at timestamptz,
  created_at timestamptz not null default now(),

  unique(label),
  foreign key (account_id) references account (id) on delete cascade,
  primary key (id)
);
