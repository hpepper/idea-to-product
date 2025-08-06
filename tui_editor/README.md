# TUI Editor

## TODO

* Remember which index entry each tab list is at.
* In the view packet, support filter by: component_id, type, style
* In the component view Be able to see which views it is in.
* Should the ID be able to be changed?
* put the vertical panes in named variables instead of in a list.
* Put the various entities in its own .rs file and have render and key handling in each file.
* Have a global key handeling that will hanlde: Menu, tabs, status
* Create tab
  * highlight the selected tab
* Add scroll bar for lists that are longer than the window
* If posible enable wrapping of long list entries.
* Allow the width of the list window to be changed, when there are long entries in list
* Design the key handling so only the parts that are active can handle inputs
  * if the selector pane, then that get its key handler called, and the detail/edit pane does not handle keys
* Manually create the list
  * ViewPackets
  * Components
  * ComponentRelations
    * Or maybe instead have a designer to the graph types, like context diagram, MSC, dependency graph etc.
* Create a selection pane and a detail pane
* Populate the detail pane from the DB
* as the user browses the detail pane, a unborderd subpane shows the details for that entry.
* enable edit
* enable add
* save back to XML
* enable customizing the color scheme.
* Add a Vocabulary tab, and enable selecting words for each viewpacket.
  * is it just two lists and the transfer it from the full list to the viewpacket list?
* Add a tab for the document introduction section
* Fix this issue when there are no entries in the selector pane

```text
thread 'main' panicked at src/work.rs:213:32:
attempt to calculate the remainder with a divisor of zero
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Design

* Put the List for each type in their own struct, so it will remember the active/selected entry, when switching between entities.

### Flow

* Read the data from .xml and put it in the mem-sqlite.
* Save every x seconds 30/60 when things have been changed.
  * save to .xml
