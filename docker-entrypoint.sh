#!/bin/sh
set -e

# Set DATABASE_URL for sqlx-cli
export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB}"

# Wait for postgres to be ready
echo "Waiting for postgres..${POSTGRES_USER} ${POSTGRES_PASSWORD} ${POSTGRES_DB} ${DATABASE_URL} ${APP_POSTGRES_DSN}"
while ! pg_isready -h postgres -p 5432 -U postgres -d postgres; do
  sleep 1
done
echo "Postgres is ready!"

# Create database if not exists
echo "Creating database if not exists..."
PGPASSWORD=$POSTGRES_PASSWORD psql -h postgres -U postgres -d postgres -c "SELECT 'CREATE DATABASE ${POSTGRES_DB}' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${POSTGRES_DB}');" || true

# Temporarily comment out backup restore section
# if [ -f "/usr/src/app/backup.sql" ]; then
#     echo "Checking if database is empty..."
#     # Create public schema if not exists
#     PGPASSWORD=$POSTGRES_PASSWORD psql -h postgres -U postgres -d ${POSTGRES_DB} -c "CREATE SCHEMA IF NOT EXISTS public;" || true
#     
#     # Check if database is empty
#     TABLE_COUNT=$(PGPASSWORD=$POSTGRES_PASSWORD psql -h postgres -U postgres -d ${POSTGRES_DB} -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" || echo "0")
#     
#     if [ "$TABLE_COUNT" -eq "0" ]; then
#         echo "Database is empty, restoring from backup..."
#         PGPASSWORD=$POSTGRES_PASSWORD psql -h postgres -U postgres -d ${POSTGRES_DB} -f /usr/src/app/backup.sql
#         echo "Data restore completed!"
#     else
#         echo "Database already has data, skipping restore."
#     fi
# fi

# Run migrations
echo "Running migrations..."
cd /usr/src/app
sqlx database create
sqlx migrate run

# Start the application
echo "Starting application..."
exec "$@"
