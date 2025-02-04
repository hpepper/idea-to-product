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
  *  for type in ['Module', 'CnC', 'Allocation']
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