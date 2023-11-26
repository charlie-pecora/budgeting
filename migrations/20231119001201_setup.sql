-- Add migration script here

create table accounts (
    id varchar primary key,
    provider varchar not null,
    type varchar not null
);

create table transactions (
    id varchar primary key not null,
    account_id varchar not null references accounts(id),
    transaction_date date not null,
    description varchar not null,
    category varchar not null,
    amount_cents integer not null, -- always in cents
    status varchar not null
);

