# Design documentation

I think that somehow I need to have a way of looking at 'Features' in the
SwArchDoc.
And right now I'm not sure how 'Feature' fits in with the Default SwArchDoc
concept. Since a feature goes accross all three view types.

* Should I have each 'feature' have its own subsection under each view type.
* Or should I have 'Feature' as a view type? (I like this idea right now).
  * Though it is not realy a seperate view type but just a different way
  * of looking at the SW.???

'Feature' would probably be more of a CnC view type. Or mostly use the CnC
view type to show the feature architecture.

## Introduction

### Purpose

## Generel

### References

#### Generating references between view packets

* Any
  * If the top Id is mentioned as a top Id in any other view, then link it as a sibling.
* Module
  * Decomposition, Uses:
    * Child: if a sub component is mentioned as the top id in a different view packet reference that view packet.
    * Parent:

#### TODO Generating references from Primary display section

### TODO Element behavior: Where does this fit in?

Does that depend on which view it is part of?

* Should the behavior be part of the Component XML.
* Should each diagram type have its own Element structure? No; Use a common 'Diagram' element.
* How do I ensure the right diagram are shown in the right place.
  * the view type pick out the Diagram list to include.
  * The Diagram has a guideline for which ViewStyle it should be a part of.
  * Diagrams are automatically included, based on the Component in the view and DiagramType.
    * Prefered; since this allows for automatic inclusion of description of sub-components that are not part of any views in their own right.
  * Or should the diagram hold a type of Viewstyle to include itself in?

Design: Viewstyle holds the list of diagrams to include:

* Advantages:
  * Exact control over which viewpackets the activity is show in.
* Disadvantages:
  * ?

Later I would like to have an XML structure so that the entries in the diagram could be references by the component id,
  like the ContextDiagram

```xml  
  <Diagram Id="" Type="">
    <ComponentId></ComponentId>
    <Description?></Description>
    <Headline><Headline>
    <Verbatim App="plantuml"></Verbatim>
  </ActivityDiagram>
```

Multiple Diagram types exist, each captured in the same XML element structure:

* Activity

Relation between the "4+1 view" and DocArch
(Bas03,ch2, p41, l1281)

* Logical = Module view
* Process = component-and-connector
* Developement = Allocation
* Physical = allocation

* Module view
  * [StateDiagrams] - A description of what state diagrams are and the concepts for creation.
  * [MessageSequenceChart] - Sequence diagrams are all about capturing the order of interactions between parts of your system.
  * [CommunicationDiagram] - model the dynamic behavior of the use case. Represent a combination of information taken from ClassDiagram, Sequence, and UseCases describing both the static structure and dynamic behavior of a system.
  * [TimingDiagram]
  * [InteractionOverviewDiagram] - Similar to the ActivityDiagram, in that both visualize a sequence of activities. 
    * The difference is that, for an interaction overview, each individual activity is pictured as a frame which can contain a nested interaction diagrams.
  * [ClassDiagram]
  * [ObjectDiagram]
  * [CompositeStructureDiagram] - Static structure diagram, that shows the internal structure of a class and the collaborations that this structure makes possible
* CnC view
  * Activity diagram
* Allocation view
  * [ComponentDiagram]
  * [PackageDiagram] - Package diagrams are often used to view dependencies among packages.
  * [DeploymentDiagram] - show the physical view of your system, bringing your software into the real world by showing how software gets assigned to hardware and how the pieces communicate(Rus06, p224)

When Building the Behavior section of the Viewpacket:

* Get the list of end point Components (put the Primary component first?)
* Get the viewpacket style.
* For each of the ComponentId' get all diagrams for the ComponentId and ViewPacketStyle.

* How to actually generate the entries?
  * Generate the PNG files(using plantuml) and put include and the rest directly into the viewpacket.tex file?

## ADL - architectural description languages

* [](http://www.ece.odu.edu/~rdmckenz/papers/SIMULATION%20A030081.pdf)

From: ACME: An Architecture Description Interchange Language; David Garlan Robert Monroe David Wile; January 14, 1997

* components
* connectors
* systems - Collection of components and connectors.
* ports - Attachment points for connectors, that connect components together.
* roles -
* representations
* rep-maps:

#### Components

represent the primary computational elements and data stores of a system.
Intuitively, they correspond to the boxes in box-and-line descriptions of software architectures. 
Typical examples of components include such things as clients, servers, filters, objects, blackboards, and
databases.

#### Connectors

represent interactions among components. 
Computationally speaking, connectors mediate the communication and coordination
 activities among components.
Informally they provide the \glue" for architectural designs, and intuitively,
they correspond to the lines in box-and-line de-
scriptions. Examples include simple forms of in-
teraction, such as pipes, procedure call, and event broadcast. But connectors may also represent more
complex interactions, such as a client-server proto-
col or a SQL link between a database and an ap-
plication.

#### Systems

represent configurations of components and connectors

#### Roles

Connectors also have interfaces that are defined by a
set of roles. Each role of a connector defines a partic-
ipant of the interaction represented by the connector.
Binary connectors have two roles such as the caller and
callee roles of an RPC connector, the reading and writ-
ing roles of a pipe, or the sender and receiver roles
of a message passing connector. Other kinds of con-
nectors may have more than two roles. For exam-
ple an event broadcast connector might have a single
event-announcer role and an arbitrary number of event-
receiver roles

## Component diagram

### CnC view diagram

Each CnC component can related to multiple other CnC components.

I would expect that a child view would explode one of the, current view packet
components, into subcomponents and display the inter-relationship of the sub
components.

A CnC view packet can then refer to a system id.

```xml
<System Id="" Name="">
  <Description></Description>
  <!-- there can be multiple ComponentId entries. -->
  <ComponentId></ComponentId>
  <!-- There can be multiple RelationId entries -->
  <RelationId></RelationId>
</System>
```
