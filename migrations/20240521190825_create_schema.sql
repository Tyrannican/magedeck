-- Add migration script here
create table if not exists cards (
    id text primary key,
    name text not null,
    set_tag text not null,
    set_name text not null,
    euro real,
    euro_foil real,
    usd real,
    usd_foil real,
    usd_etched real,
    tix real,
    cardmarket text,
    cardhoarder text,
    tcgplayer text
);

create index if not exists idx_name on cards(name);
create index if not exists idx_set on cards(set_tag);
create index if not exists idx_set_name on cards(set_name);
