create table namespace (
  id uuid default uuid_generate_v4(),
  label text not null,
  account_id uuid not null,
  enabled boolean not null default false,
  created_at timestamptz not null default now(),

  unique(label),
  foreign key (account_id) references account (id) on delete cascade,
  primary key (id)
);
