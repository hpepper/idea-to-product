# idea-to-product

Tools for capturing information for getting from idea to product

An anotated version of the .dtd is in the Documentation dir of the gui vibecoding dir. (for the llm agent to read, during develeopment)

Using it in your own project.

1. cp ../idea-to-product/itp.xml .
2. ln -s ../idea-to-product/Makefile
3. ln -s ../idea-to-product/itp2doc.pl

To validate the XML structure:

- clear; xmllint --valid --noout --dtdvalid Itpp/ideatoproduct.dtd Itpp/Documentation/itp_itpp.xml

When adding a new component:

1. Create the component.
2. Add the relevant ModuleRelation
3. If the the component has children, then create a view packet for the component.
   - A view packet, for Module decomposition is only relevant if the if a sub-module in the view packet has sub-modules.
     Otherwise the list of sub-modules are in the primary presentation.

When implementing a new view packet style:

1. itp2doc.pl: Add the new entry to GenerateSoftwareArchitectureDocumentation().
   - e.g. GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szAllocationTypeName, $f_szImplementationStyleName);
2. LaTeX_SwArcDoc.tmpl: Add code to retrieve the data (e.g. Type: Allocation Style: Implementation :
   - The array name is automatically generated in GenerateViewPacketByTypeAndStyle(): $refhSwArcHashList->{"ar${szViewPacketTypeName}${szViewPacketStyleName}ViewPackets"} = \@arViewPacketFileList;
{ foreach my $szModuleFileName (@arAllocationImplementationViewPackets) {
    $OUT .= "\\input{$szModuleFileName}\n";
     }
     }

Adding a ContextModel to a Viewpacket:

1. Create the required component relations, for each link:
   - ComponentAId: Component Id of 'TheWork'
   - ComponentBId: Component Id of the related entity.
   - Style: ContextModel
2. create a view packet for the ContextModel
   - ViewPacketType: Sub
   - ViewPacketStyle: ContextModel
3. Add Id of the Context viewpacket to the ContextModelId of the 'master' viewpacket

## Introduction to SAD

SAD takes the approach of looking at one aspect of a entity of a product, like how to install it is one aspect.
A product consists of many entitys; like DB, logging, user authentication etc.
SAD splits each entity of
SAD splits the description each entity into a number subentitys(viewpackets); like

- what does the entity consist of
- what does the entity depend on
- what other entitys does this entity communicate with
- how is this entity installed
- where is this entity deployed; like in a k8s cluster, in an EC2, lambda etc.

TODO why the viewpacket approach, where the description of a component is split out over multiple viewpackets, and not everything about the enity in one viewpacket?

- TODO you would still need to have the sections, like breakdown and once you have the breakdown, it is easier for that broken down entity to be described in the various ways at that level, so that you would not have the gamesever broken down into sub modules and
  when you break down the game server into its components, then it is easier to describe each of those broken down components than have everything in the same section.

TODO why the break down into the styles?

The documentation is split into three major types:

- Module - look at the product as as enities can be implemented
  - (from high level to detailed entities that can be implemented by a single team)
  - The module part are split into a number of sections called styles
- Component and Connectors - how each entity communicates with other entities TODO rework this, make it logical like in philosophy
  - Component - TODO
  - Connector - TODO
- Allocation - TODO catch-all

Each type is split into a number of styles:

- Module styles
  - decomposistion - TODO
  - uses - TODO
  - layers - TODO
- Component and connectores
  - pub-sub - TODO
  - client-server - TODO
  - ... TODO others?
- Allocation
  - Deployment - describes where a component is running.
    - e.g. in a k8s cluster or on an EC2 instance, etc.
  - Installation - describes how the component is transferred to the target.
    - where the target can be a k8s cluster, an ec2 instance or another cloud service.
  - Testing - how to test the component
    - Both during build and also after deployment.

### Viewpackets - TODO short description

A view packet is a way to hold the relevant information for an entity.

For example the EdgeConnector entity would be describe in the following five viewpackets:

- Module - Decomposition - where in the hieraki(TODO find a better word for this) that the EdgeConnector is part of.
- Module - Uses - what other entities the EdgeConnector requires to run.
- CnC - client-server : how the client connects to it
- CnC - publish-subscribe : TODO the EdgeConnector publishes to the send channel and subscribes to the receive channel.
- CnC - pub-sub : telemetry?
- Allocation - Deployment : where in the system the EdgeConnector is running.
- Allocation - Installation : how the EdgeConnector is delivered to the deployment it is running in.

TODO also explain about how the Threatmodel can be buildt from this archecture, probably via CNC and install.

#### General content of a ViewPacket

- Headline
- Primary display
- Context diagram

## Adding to the document

### What not to include

- things that change often and does not affect other teams/modules, need not/should not be documented in the architecture document.
