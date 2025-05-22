#!/bin/sh
set -e

# Wait for postgres to be ready
echo "Waiting for postgres..."
while ! pg_isready -h postgres -p 5432 -U $POSTGRES_USER -d $POSTGRES_DB; do
  sleep 1
done
echo "Postgres is ready!"

# Run migrations
echo "Running migrations..."
sqlx database create
sqlx migrate run

# Restore data if backup file exists
if [ -f "/usr/src/app/backup.sql" ]; then
    echo "Restoring data from backup..."
    PGPASSWORD=$POSTGRES_PASSWORD psql -h postgres -U $POSTGRES_USER -d $POSTGRES_DB -f /usr/src/app/backup.sql
    echo "Data restore completed!"
fi

# Start the application
echo "Starting application..."
exec "$@" 
