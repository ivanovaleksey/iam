create table identity (
  provider uuid,
  label text,
  uid text,
  account_id uuid not null,
  created_at timestamp not null default now(),

  foreign key (account_id) references account (id) on delete cascade,
  foreign key (provider) references namespace (id) on delete cascade,
  primary key (provider, label, uid)
);

create type identity_composite_pkey as (
    provider uuid,
    label text,
    uid text
);
