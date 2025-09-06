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

* cargo test
* cargo test --test test_xml_load_and_dump -- --nocapture
  * test that specific test and show output.

## Design

* The XML is loaded into an in-memory sqlite db.

* The team xml becomes
  * team table
  * teammember_lut - team id to team member look-up-table

### For the Allocation - Assignment viewpacket style

* Primary display
  * This is a list of components that a team is responsible for.
  * followed by the team description and member list.
* Related views
  * only the sibling list: for each component_id reference all viewpackets that describes that component.


* In the viepacket there is generated an alphabetic table with the component name and the component description.
* Each viewpacket that has a ComponentID in an 'Allocation - Assignment viewpacket' must reference that assignment viewpacket as a sibling.
* The 'Allocation - Assignment viewpacket' must reference each viewpacket that describe any ComponentID in the packet as a sibling
  * For each ComponentID in the Assignment viewpacket we must create a sibling reference to all viewpackets describing the ComponentID.

* The viewpacket has a single TeamID xml entry, only for the Allocation - Assignment viewpacket type.

* Each component has an optional single TeamID, this is written into the component_team_lut
  * the lut should probably be writen as unique but gracefully ignore duplicates.
  * Actually I do not have to ensure the above since there will only exist one instance of each component, so there can never exist multiple component id.

to make the above unique

```sql
CREATE TABLE component_team_lut (
    component_id INTEGER,
    team_id INTEGER,
    UNIQUE(component_id, team_id)
);

INSERT OR IGNORE INTO component_team_lut (component_id, team_id)
VALUES (?, ?);
```

#### Loading the component into the DB

* Load the component normal, I actually don't even need a lut for the component_id - team_id, since there can only exist a singlre relation between the two.

#### Loading the team into the DB

* Load all data except the member list into the team table
* Load each member into the team_to_member_lut
  * team_id
  * member name
  * member role

#### Generate the Assignment viewpacket

* Get the team id from the viewpacket.
* The component list table
  * get a list of all components that has that team id, sorted by the component name
  * popluate the table with the component name and the component description
* The team description and member list
  * Get the team from the team table for the team_id
  * get a list of members from the team_to_member_lut sorted alphabetically by member name

### Convert from old to new version of the xml file

Read in the version, then in each read function, do a match on the version, and default to read the newest version.

Ammend the the data read so it fits the current db structure and write the data to the table.
Once all data has been read in, then dump to a new file in the current version format.

## TODO

* Load includes into DB
* Load each includes (mark them as loaded when the include file is loaded into DB)
  * the next level includes should be marked as loaded.
  * If I try to include a fileid i have in the db I should fail.
* Add support for converting from old xml format to new.
* Make name of components unique so I can use it for reference in the component selector list.
