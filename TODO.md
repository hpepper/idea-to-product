* TODO V Get Title/Name and Summary into relations tables. In GenerateComponentDiagramAndLegendRecursively change the GetSubComponentList to GetRelationsIdList and then work with the relationslist.
* TODO V automatically generate ViewPacket references to other viewpackets where the focus component also is the focus component.
* TODO V Support Module Layers.
* TODO V Include requirement references, but how and where. A component might
  have a req for how data on an interface must look. which the other end of the
  connection must honor.
* TODO X next step: Update Graphviz_componentDiagram.tmpl to handle the lists.
* TODO C Add the rest of the sections to 'Other Information' like Use case etc.
* TODO C Support 'Behavior' entries. Initially use plantuml text syntax, generate a picture and include the picture in the tex file. Possibly include caption and maybe description?
* TODO C Support 'UseCase(s)' Both text and graphics.
* TODO C Have the component id, present, at least, in the overview sections(like the chapter start of the module decomposition chapter) to make it easier to quickly lookup the component in the xml file. Preferably also have the ID in the the graphical presentation.
* TODO C if a referenced component does not have it's own view packet, then fully describe it under this view packet(What if the component is referenced in multiple view packets? should it be fully described in all, should one instance have the full description and the rest just reference it?) Update LaTeX_ViewPacket.tmpl
* TODO C Create the the Viewpacket related views automatically. Probably just run through all the view packets and create a hash(key:component id) with a list of the View ID that is referring to that component.
* TODO C Finish the Viewpacket operations.
* TODO V Change the include-file names to Wiki format, like 'ViewPacketModuleSomething1' instead of 'viewpacket_module_something_1'
* TODO V Change the ComponentDiagram to Component, like in XxArchDoc
* TODO V Make the SIPOC table a long table, so that it does not get moved? or maybe make it sub paragraphs.
* TODO V Make the links in the PDF active, so I can click them to navigate the document.
* TODO V Allow for sections to be included into the overview chapter.
* TODO V Generate a ComponentDiagram at the beginning of each View chapter, that should include the entire tree for this chapter.
* TODO V  Also do a legend for the thing, with references.
* TODO V can I automatically find siblings, by searching all viewpackets looking for other viewspackets where the current ComponentId is also the ComponentId
* TODO V Search for the Parent Id ?
* TODO V At the top of each view packet, include a description of the top component.
* TODO V Log errors to a log file, to make it simpler to troubleshoot and recognize that there is a problem.
* TODO V Add the HW env/overview thing, but where?
* TODO V Make it fill out to an A4 page.
* TODO V Have an appendix where every component is fully described, meaning all aspects of the component is described, included all references to everywhere, the component has been mentioned/described.
* TODO V Should the Component have a Description tag with 'Aspect' attribute, so it is displayd in the description section for that ViewStyle?
* TODO V check that a child, parent or sibling viewpacket id isn't the same as the current id: Deep recursion on subroutine "main::GenerateViewPacketRecusrsively" at ../idea-to-product/itp2doc.pl line 900.
* TODO N  The above would also allow me to calculate the amount of view packet, I can simply use the Global viewpacket counter.
* TODO N If a view packet is a child packet, then make it a subsection(somehow put the level in the call to the template and let an if-else set select the propper heading.)
* TODO N What should I do about the, very big, Context models, they are unreadable.
* TODO N Don't do Elements and Properties subsubsection, if there are no sub modules.

Generate the generic SIPOC
 (List of lists)
~~~
<start with the decsription>
 - Supplier
   - entry
   - entry2
 - Input
 ...
~~~

* Include the SIPOC in the FIP (use subsubsection for each sipoc (headline)
* Put SIPOC generation in the Makefile
* Generate the FIP.

ComponentModel
1. Find an example where I need the component model.
2. Then code for that to work.
3. Then find the next example and code for that.

Is this giving us active maps: http://www.graphviz.org/Gallery/undirected/networkmap_twopi.gv.txt

* TODO V At first the UML generation will be simply the text included in the section of the packet view. Later it should be XML defined so that the Component IDs can be re-used.
* TODO V Do Use Case in XML and be able to generate a graph and text representation of it.

~~~
<!-- Sequence diagrams - should just be done in text
@startuml
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response

Alice -> Bob: Another authentication Request
Alice <-- Bob: another authentication Response
@enduml
-->
~~~
