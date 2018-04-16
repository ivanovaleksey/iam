create table identity (
  provider uuid,
  label text,
  uid text,
  issuer_id uuid not null,
  account_id uuid not null,
  issued_at timestamp not null default now(),

  foreign key (issuer_id) references account (id) on delete cascade,
  foreign key (account_id) references account (id) on delete cascade,
  foreign key (provider) references namespace (id) on delete cascade,
  primary key (provider, label, uid)
);
