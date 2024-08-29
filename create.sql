BEGIN TRANSACTION;
    CREATE TABLE seen_news (
        id TEXT PRIMARY KEY NOT NULL
    );
/*
    CREATE TABLE seen_events (
            id INTEGER PRIMARY KEY NOT NULL,
            event_id TEXT NOT NULL
    ); */
COMMIT;
