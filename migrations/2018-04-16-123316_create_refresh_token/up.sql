create table refresh_token (
  account_id uuid,
  algorithm text not null,
  keys bytea[] not null,
  created_at timestamptz not null default now(),

  foreign key (account_id) references account (id) on delete cascade,
  primary key (account_id)
);
