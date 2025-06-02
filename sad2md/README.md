# SAD to MD

## Introduction

### References

* Bas24 - Software architecture in practice, 4th edition

### Creating this project

* cargo new sad2md
* cd sad2md
* ~/Dropbox/fix_rust_build_dir.sh
* cargo add xmltree
* cargo add rusqlite --features bundled
  * bundle compiles the sqlite code into the binary

### Build

* cargo run test/test_sad.xml && cat draft_architecture.md
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

## Design

### Behavior

* Has the following entries:
  * Description - One entry.
  * DiagramId* - 0 or more.

* Have a Diagram Entity, for holding diagrams
  * to be used in e.g. Behavior
  * for e.g. MSCs
  * TODO is a Diagram lke a mini viewpacket?

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

TODO Primary representation graphical

Do not print the inital one like in the textual
 but in the recursive function, print a and b name in the same line
 create the diagram nodename by removing all spaces from the titel of the a and b component
then have the mermaid lead-in and lead out in the parrent function( I thing iew packet riender fnction.)

TODO the related view

* The order number is used used within a style, and is the last part of the section name(z.y.z)

#### ComponentRelation - ConnectionType

* [](https://en.wikipedia.org/wiki/Message_sequence_chart)

* Id - UUID
* SortOrder - for used within the same diagram. Can be left empty if not required.
* ComponentAId - Left component
* ComponentBId - Right componnet
* Key - Used for getting all relations for a specific diagram.
* PropertyOfRelation - TODO what is this for?
* RelationText - for MSC and possibly also ContextDiagrams?
  * TODO should this be a reference instead, so that it can reference e.g. a protocol.
* ConnectionType
  * MSC connection types
    * Call - Sending a message.
    * Response - return a message.
    * InstanceCreation - Create a line that did not exist from the start.
    * InstanceDestruction - End the line.
  
#### ViewPacket Behavior

* For now the Behavior element is outside the viewpacket and references the viewpacket it belongs to.
  * Later the Behavior element could be defined inside the ViewPacket element.
    * Choosing outside to make the code of reading it in from the XML simpler, even though writing the xml is harder.

## Software Architecture Documentation

### Introduction to Software Architecture Documentation

Software Architecture Documentation(SAD) is there to give stakeholders a way to understand the product without having to go through and learn the source code.

SAD takes the approach of looking at one aspect of a entity of a product, like how to install it is one aspect.

* Component - principal units of computation(Bas24, p6), like services, peers, clients, servers, filter etc.
* Connectors - communication vehicles among componets, like call-return, pipes etc.(Bas24,p6)
* Element - a unit of software architecture that participates in the system’s structure and behavior. It can be a component, connector, interface, data element, or even a configuration.
* Product - the complete solution available to the customers.

A product consists of many entities; like DB, logging, user authentication etc.
SAD splits each entity of
SAD splits the description each entity into a number subentitys(viewpackets); like 

* what does the entity consist of
* what does the entity depend on
* what other entitys does this entity communicate with
* how is this entity installed
* where is this entity deployed; like in a k8s cluster, in an EC2, lambda etc.

TODO why the viewpacket approach, where the description of a component is split out over multiple viewpackets, and not everything about the enity in one viewpacket?

* TODO you would still need to have the sections, like breakdown and once you have the breakdown, it is easier for that broken down entity to be described in the various ways at that level, so that you would not have the gamesever broken down into sub modules and
when you break down the game server into its components, then it is easier to describe each of those broken down components than have everything in the same section.

TODO why the break down into the styles?

The documentation is split into three major types:

* Module - look at the product as as enities can be implemented
  * (from high level to detailed entities that can be implemented by a single team)
  * The module part are split into a number of sections called styles
* Component and Connectors - how each entity communicates with other entities TODO rework this, make it logical like in philosophy
  * Component - TODO
  * Connector - TODO
* Allocation - TODO catch-all

Each type is split into a number of styles:

* Module styles
  * decomposistion - TODO
  * uses - TODO
  * layers - TODO
* Component and connectores(CnC) - focus on the way the elements interact with each other at runtime.
  * pub-sub - TODO
  * client-server - TODO
  * ... TODO others?
* Allocation
  * Deployment - describes where a component is running.
    * e.g. in a k8s cluster or on an EC2 instance, etc.
  * Installation - describes how the component is transferred to the target.
    * where the target can be a k8s cluster, an ec2 instance or another cloud service.
  * Testing - how to test the component
    * Both during build and also after deployment.

### Viewpackets - TODO short description

A view packet is a way to hold the relevant information for an entity.

For example the EdgeConnector entity would be describe in the following five viewpackets:

* Module - Decomposition - where in the hieraki(TODO find a better word for this) that the EdgeConnector is part of.
* Module - Uses - what other entities the EdgeConnector requires to run.
* CnC - client-server : how the client connects to it
* CnC - publish-subscribe : TODO the EdgeConnector publishes to the send channel and subscribes to the receive channel.
* CnC - pub-sub : telemetry?
* Allocation - Deployment : where in the system the EdgeConnector is running.
* Allocation - Installation : how the EdgeConnector is delivered to the deployment it is running in.

TODO also explain about how the Threatmodel can be buildt from this archecture, probably via CNC and install.

#### General content of a ViewPacket

* Headline
* Primary display
* Context diagram

## Adding to the document

### What not to include

* things that change often and does not affect other teams/modules, need not/should not be documented in the architecture document.