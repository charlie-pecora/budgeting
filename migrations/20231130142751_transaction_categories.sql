create table categories (
    id varchar primary key not null,
    name varchar not null
);

create table sources (
    id varchar primary key not null,
    name varchar not null
);

drop table if exists transaction_categories;
create table transaction_categories (
    transaction_id varchar not null references transactions(id),
    category_id varchar not null references categories(id),
    source_id varchar not null references sources(id),
    created_at timestamp,
    is_active boolean not null
);
