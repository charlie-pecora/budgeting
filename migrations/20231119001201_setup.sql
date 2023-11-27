-- Add migration script here

create table banks (
    id varchar primary key not null,
    name varchar not null
);

create table account_types (
    id varchar primary key not null,
    name varchar not null
);

create table accounts (
    id varchar primary key,
    name varchar not null,
    bank_id varchar not null references banks(id),
    type varchar not null references account_types(id)
);

create table transactions (
    id varchar primary key not null,
    account_id varchar not null references accounts(id),
    transaction_date date not null,
    description varchar not null,
    amount_cents integer not null, -- always in cents
    status varchar not null
);

create table transaction_categories (
    transaction_id varchar not null references transactions(id),
    category varchar not null,
    source varchar not null,
    created_at timestamp,
    active boolean not null
);
