PRAGMA foreign_keys=OFF;

BEGIN TRANSACTION;
    CREATE TABLE IF NOT EXISTS seen_news (
        id INT PRIMARY KEY NOT NULL,
        news_id TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS seen_events (
        id INT PRIMARY KEY NOT NULL,
        event_id TEXT NOT NULL
    );
COMMIT;
