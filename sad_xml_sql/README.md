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

## Created

* cargo lib sad_xml_sql ?
* cargo add simple_xml_builder
* cargo add xmltree
* cargo add rusqlite

## Testing

* cargo test --test test_xml_load_and_dump -- --nocapture

## TODO

* Make name of components unique so I can use it for reference in the component selector list.
