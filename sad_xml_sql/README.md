# SAD XML SQL lib

## to use

### Cargo.toml

```rust
[dependencies]
sad_xml_sql = { path = "../sad_xml_sql" }
rusqlite = { version = "0.35.0", features = ["bundled"] }
```

### main.rs

```rust
use sad_xml_sql::db_population;
use sad_xml_sql::db_retrieval;
use sad_xml_sql::models;

use rusqlite::Connection;
```

* fn main()

```rust
    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    populate_db(&db_conn, filename);
```