If a sub-component is included in the primary presentation of the viewpacket, and that sub-component is not referenced as primary in any view packet, then include the full sub-component description
 in this view, as if it was the primary component.
This is only for sub-components, not sub-sub-components etc.
e.g. PrimaryComp -> SubComp -> SubSubComp
If the 'SubComp' does not have its own ViewPacket, then include all the (relevant) information of the SubComp in this ViewPacket.
? Should this be recursive? So sub-sub-components also gets described?


The 'Reference' in the Component Legend should refere to the view packets where the Component has a primary description(including the ViewPacket where it is a sub-component with a primary description).


New View Type: Production (How to produce the product) With flow diagrams etc.

Where should 'Tech lead' be noted? in the 'Work allocation' view?


Possibly use later:


<ProjectObjective Id="" Project="">
  <ProjectObjectiveActionId></ProjectObjectiveActionId>
  <SixSigmaCardinalObjectiveId></SixSigmaCardinalObjectiveId>
  <MarketSegment></MarketSegment>
  <ObjectiveDirectionId></ObjectiveDirectionId>
  <Measure></Measure>
  <Target></Target>
  <Deadline></Deadline>
  <State></State>
  <Priority></Priority>
  <Comment></Comment>
</ProjectObjective>

<ProjectObjectiveAction Id="" Name="">
  <Description></Description>
</ProjectObjectiveAction>

SixSigmaCardinalObjective Id="" Name="">
  <Description></Description>
</SixSigmaCardinalObjective>

<ObjectiveDirection Id="" Name="">
  <Description></Description>
</ObjectiveDirection>


## User: Adding support for Diagrams
===========================
1. Add 'Diagram' element to itp.xml and ideatoproduct.dtd
2. Add 'DiagramId' to the 'ViewPacket' element in ideatoproduct.dtd
3. Add the IncludeDiagramIfExists function to itp2doc.pl
4. Add the IncludeDiagramIfExists call to function
    - GenerateSoftwareArchitectureDocumentation
    - GenerateViewPacketRecusrsively
    - in itp2doc.pl
5. Add the GenerateDiagram function to itp2doc.pl

## Implementation of the Diagram support.

GenerateSoftwareArchitectureDocumentation()
 -> IncludeDiagramIfExists()
 -> GenerateUmlDiagram()
   Inserts the diagram, in LaTeX, using the template: LaTeX_componentDiagram.tmpl
   the .ps file is generated using plantuml.

 The Diagram filename is then included in 'LaTeX_ViewPacket.tmpl'
