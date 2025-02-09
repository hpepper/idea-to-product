# SAD to MD

## Introduction

### Creating this project

* cargo new sad2md
* cd sad2md
* ~/Dropbox/fix_rust_build_dir.sh
* cargo add xmltree
* cargo add rusqlite --features bundled
  * bundle compiles the sqlite code into the binary

### Build

* cargo run test/test_sad.xml
* xmllint --valid  test_sad.xml  > t 2>u; head u

### Overview

* read the xml and write the data into the in-memory db
* then generate the MarkDown file by extracting the data from the database.

* Read the view packet list
  * for type in ['Module', 'CnC', 'Allocation']
    * for style in StyleListForType(type)
      * for viewPacket in getSortedListOfViewPackets(type, style)
        * renderViewPacket(viewPacket)

* The ViewPacket references the ComponentId and ContextDiagramId
  * ComponentId - Id of the Component.
  * ContextDiagramId - Id of the starting ComponentRelation Id
    * Do I need the `<Style>ContextModel</Style>` ?
    * How do I build the context diagram, I have a starting point in the relation, but what helps me know what else to pick?
    Should I do an sql  select * FROM ComponentRelation WHERE ComponentAId == 1 AND style == "ContextDiagram"
    Or should the ComponentRelation have a diagram key that is unique for the diagram, and that key is the one noted in the ContextDiagramKey(TODO rename from ContextDiagramId) ?

* TODO how do I define an MSC in the sad.xml?

* ComponentRelation
  * The ComponentAId is 'The Work' in a context diagram.
    * it is known that it is a contextdiagram, from the referance in the viewpacket.

## Implementation notes

### How viewpacket is being used for rendering

these fielads are used for...

* ComponentId
  * Look up the Component with that ID to get the details for the component to display.
  * Combined with the PrimaryDisplayKey to lookup component tree in the ComponentRelation
    * for generating the primary display.
    * I can have a viewpacket that have a 4 level deep primary display.
    * I can have another viewpacket that show a component that is at e.g. level 3 of the top viewpacket
      * The PrimaryDisplayKey in this new viewpacket will be the same as the previous view packets.
      * This works because the new viewpacket just point at a component further down the tree, and the search does not go up the tree only down.
  * Combined with ContextModelKey to lookup component tree in the ComponentRelation
    * for generating the Context diagram.

TODO Primar  representation graphical

Do not print the inital one like in the textual
 but in the recursive function, print a and b name in the same line
 create the diagram nodename by removing all spaces from the titel of the a and b component
then have the mermaid lead-in and lead out in the parrent function( I thing iew packet riender fnction.)