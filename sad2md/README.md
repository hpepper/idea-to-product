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

* cargo run ../sad_xml_sql/tests/test_sad.xml && cat draft_architecture.md
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

### XML updates

- grep "ComponentRelation Id" ../sad_xml_sql/tests/test_sad.xml | sort
- grep "Component Id" ../sad_xml_sql/tests/test_sad.xml | sort

## Design

### Behavior

* Has the following entries:
  * Description - One entry.
  * DiagramId* - 0 or more.

* Have a Diagram Entity, for holding diagrams
  * to be used in e.g. Behavior
  * for e.g. MSCs
  * TODO is a Diagram like a mini viewpacket?

### Assignment

#### Create links from Viewpackets to the assignment viewpacket for a given TeamID

* In the non-assignment viewpacket get the ComponentID
* get component with ID
* Look up a viewpacket with component TeamID

### Generating deployment diagrams

- Prompt 1
  - The end goal is to generate a deployment diagram, wich can depict comoponents,
  - some components inside other components
  - some components containing multiple components.
  - in rust how to read the list component relations for a diagram and generate a mermaid diagram
- Prompt 2
  - In rust how to create a tree the holds the relations between componentrelations of ComponentAId and ComponentBId, where the top level contains all componentRelations where ComponentAId is not ComponentBId in any other relations

See #### ComponentRelation - ConnectionType

- get all ConnectionType is Contains for Key 

## Implementation notes

### How viewpacket is being used for rendering

these fields are used for...

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
  * Deployment
    * Contains - creates a subgraph
      * I guess I need to go and find the root components, so I guess I need to build a tree
    * Connect - line attachment
    * Allign - for the ~~~ thing

#### ViewPacket Behavior

* For now the Behavior element is outside the viewpacket and references the viewpacket it belongs to.
  * Later the Behavior element could be defined inside the ViewPacket element.
    * Choosing outside to make the code of reading it in from the XML simpler, even though writing the xml is harder.

## Software Architecture Documentation

### Introduction to Software Architecture Documentation

Software Architecture Documentation(SAD) is there to provide stakeholders with an understanding of the product without having to go through and learn the source code.

The goal of documenting an architecture is also to write it down so that others can successfully use it, maintain it, and build a system from it(Cle02,p20).

#### SAD Vocabulary

* Component - principal units of computation(Bas24, p6), like services, peers, clients, servers, filter etc.
* Connectors - communication vehicles among componets, like call-return, pipes etc.(Bas24,p6)
* Element - a unit of software architecture that participates in the system’s structure and behavior.
  * It can be a component, connector, interface, data element, or even a configuration.
* Entity - can be a Component or a Module.
* Module - TODO
* Product - the complete solution available to the customers.
  * A product consists of many entities; like DB, logging, user authentication etc.

#### What are view packets / The viewpacket approach

A view packet is the smallest cohesive bundle of documentation that you would give to a stakeholder, such as a development team or a subcontractor(Cle03, ch6.p139).

It’s is a set of diagrams, descriptions, and rationale that explain one slice or perspective of the system.

SAD uses ViewPackets to look at one aspect of an entity at a time.

TODO clean up

Some examples of aspects:

* what does the entity consist of
* what does the entity depend on
* what other entitys does this entity communicate with
* how is this entity installed
* where is this entity deployed; like in a k8s cluster, in an EC2, lambda etc.

A viewpacket will include refrences to the other relevant viewpackets.

Viewpackets are used to split the documentation into managable and easily digestible sections.

TODO why the viewpacket approach, where the description of a component is split out over multiple viewpackets, and not everything about the enity in one viewpacket?

* TODO you would still need to have the sections, like breakdown and once you have the breakdown, it is easier for that broken down entity to be described in the various ways at that level, so that you would not have the gamesever broken down into sub modules and when you break down the game server into its components, then it is easier to describe each of those broken down components than have everything in the same section.

#### At the high level there are three types of viewpackets

The documentation is split into three major types:

* Module - look at the product as as enities can be implemented
  * (from high level to detailed entities that can be implemented by a single team)
* Component and Connectors(CnC) - focus on the way the elements interact with each other at runtime.
* Allocation - how the relationship between the software elements and the elements in one or more external environments in which the software is created and executed.

#### The styles of viewpackets

Each type is split into a number of styles:

* Module styles
  * decomposistion - TODO presents the functionality of a system understanable modules that grows ever more detailed as you dive deeper.
  * uses - TODO Show what a module requires to operate correctly.
  * layers - TODO used to describe the allowed-to-use relation in a restricted fashion between groups of modules called layers(Cle11,p65)
* Component and connectores(CnC)
  * pub-sub - TODO components interact via announced events.
  * client-server - TODO components interact by requesting services of other components. The essence of this style is that communication is typically paired and initiated by the client(,99).
  * Peer-to-peer - components directly interact as peers by exchanging services.
  * ... TODO others?
* Allocation
  * Deployment - describes where a component is running. In the deployment style it is shown which logical groups components are assigned to(Cle11, p191)
    * e.g. in a k8s cluster or on an EC2 instance, etc.
  * Installation - describes how the component is transferred to the target.
    * where the target can be a k8s cluster, an ec2 instance or another cloud service.
  * Testing - how to test the component. Show what major testing is done to significant modules.
    * Both during build and also after deployment.
  * Work assignment - Show which signinicant modules belong to what groups. (Cle11, p190).

#### Example of a Viewpacket split

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

* Primary presentation
  * Shows the elements and the relationship among those that populate the view(Bas03, p206)
  * Both graphical and textual
* Context diagram
  * TODO Shows how the component depicted in this view packet relates to its environment(Bas03, p208).
* Element catalog
  * Summarises at least those elements and relations depicted in the primary presentation, and perhaps others that are not in the primary presentation(Bas03, p207).
* Element behavior
  * Some elements have complex interactions with their environment, which will be descriped here.
* Related view packets
  * (Bas03, p207).
  * Parent* Variability guide
  * What can be cusotmized to allow this element to be used in a different way than how it works in this description(cel11,23)
* Architecture background
  * explain to someone why the design is as it is and to provide a convincing agument that it is sound(Bas03, p208)
* Design rationale
  * The architect explains why the design decisions reflected in the view packet were made and gives a list of rejected alternatives and why they were rejected.
* Result of Analysis
  * The architect should document the results of analyses that have been conducted, such as the results of performance or security analysis or a list of what would have to change in the face of a particular kind of system modification.
* Assumptions
  * The architect should document any assumptions he or she made when crafting the design. Assumptions are usually about either environment or need.
* Glossary of terms
  * Words used in the view, with a brief description of each(Bas03, p208).
* Other information
  * Content will vary according to the standard practices of your organization (Bas03, p208).

* Elements and their properties
  * names each element in the view packet and lists the properties of that element.
* Relations and their properties
  * the specific relation type(s) that are depicts among the elements in this view.
* Element interfaces section
  * An interface is a boundary across which elements interact or communicate with each other.
* Variability guide
  * What can be cusotmized to allow this element to be used in a different way than how it works in this description(cel11,23)
* Architecture background
  * explain to someone why the design is as it is and to provide a convincing agument that it is sound(Bas03, p208)
* Design rationale
  * The architect explains why the design decisions reflected in the view packet were made and gives a list of rejected alternatives and why they were rejected.
* Result of Analysis
  * The architect should document the results of analyses that have been conducted, such as the results of performance or security analysis or a list of what would have to change in the face of a particular kind of system modification.
* Assumptions
  * The architect should document any assumptions he or she made when crafting the design. Assumptions are usually about either environment or need.
* Glossary of terms
  * Words used in the view, with a brief description of each(Bas03, p208).
* Other information
  * Content will vary according to the standard practices of your organization (Bas03, p208).

#### Relation between the "4+1 view" and DocArch

(Bas03, p41)

* Logical = Module view
* Process = component-and-connector
* Developement =  Allocation
* Physical = allocation

#### Stakeholders and the Architecture Documentation view packets the stakeholders might find most useful

(Bas03, p205)

Source: (Cle11,p326)

Stakeholder           | M-Decomp | M-Uses | M-Layer | M-Gen  | C&C | A-Depl | A-Imp | A-inst | A-work
--------------------  | -------- | ------ | ------- | ------ | --- | ------ | ----- | ------ | ------
Analyst               | d        | d      | d       | s      | s   | d      |       | s     | .
Architect             | d        | d      | d       | d      | d   | d      | s     | d     | s
Customer              |          |        |         |        |     | o      |       |       | .
Dev team              | d        | d      | d       | d      | d   | s      | s     | d     | .
End user              |          |        |         |        | s   | s      |       | o     | .
Infrastructure supp   | s        | s      |         |        | s   | d      | d     | o     | .
Maintainer            | d        | d      | d       | d      | d   | s      | s     |       | .
New stakeholder       | x        | x      | x       | x      | x   | x      | x     | x     | x
Prod line app bld     | d        | d      | o       | s      | s   | s      | s     | s     | .
Projet Manager        | s        | s      | s       |        |     | d      |       |       | d
Test and Integration  | d        | d      | d       | d      | s   | s      | s     | d     | .

* d: detailed information
* s: some details
* o: overview information
* x: anything
* .: ignore, for formating rule purposes.

See also Choosing the Views(Cle11, p315).

## Adding to the document

### What not to include

* things that change often and does not affect other teams/modules, need not/should not be documented in the architecture document.

## Using the _sad.xml file

### Introduction to _sad.xml

#### Purpose of _sad.xml

#### Overview of _sad.xml

### Fill in usage

* Create a Viewpacket `<ViewPacket Id="1202" ViewType="Module" ViewStyle="Uses" SortOrder="2">`
  * ComponentId - the id of the component you are focusing on.
  * PrimaryDisplayKey - unique key.
* ComponentRelation
  * ComponentAId - the id of the component in focus.
  * ComponentBId - the id of the component that ComponentAId uses.
  * Key - the PrimaryDisplayKey of the view packet.
  * Style - Set to 'Uses' if you want to be able to create a used-by viewpacket for the ComponentBId.

* For the UsedBy view packet
  * Create a Viewpacket `<ViewPacket Id="1302" ViewType="Module" ViewStyle="UsedBy" SortOrder="3">`
    * ComponentId - the id of the component you want to show all the components that uses it.
    * PrimaryDisplayKey - empty(not used). TODO maybe this could be used in the search instead of the hard-coded 'Uses'
  * ComponentRelation - none defined, the code will search for all ComponentRelation where ComponentId is in ComponentBId and the Style is 'Uses'.
