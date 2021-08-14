#!/bin/bash

set -x
set -eo pipefail

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=passw00t}"
DB_NAME="${POSTGRESS_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: `psql` is not installed"
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: `sql` is not installed"
    echo >&2 "Use:"
    echo >&2 "  cargo install --version=0.5.5 sqlx-cli --no-default-features --features postgres"
    echo >&2 "to install it"
fi

if [[ -z "$SKIP_DOCKER" ]]; then
    docker stop postgrestest || true
    docker rm postgrestest || true

    docker run \
        --name postgrestest \
        -e POSTGRES_USER=$DB_USER \
        -e POSTGRES_PASSWORD=$DB_PASSWORD \
        -e POSTGRES_DB=$DB_NAME \
        -p "$DB_PORT":5432 \
        -d postgres \
        postgres -N 1000
fi

export PGPASSWORD="$DB_PASSWORD"
until psql -h "localhost" -U "$DB_USER" -p "$DB_PORT" -d "postgres" -c '\q'; do
    >&2 echo "Waiting for postgres"
    sleep 1
done

echo "Database is up on port port $DB_PORT"

export DATABASE_URL=postgres://$DB_USER:$DB_PASSWORD@localhost:$DB_PORT/$DB_NAME
sqlx database create
sqlx migrate run
