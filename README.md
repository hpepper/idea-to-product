idea-to-product
===============

Tools for capturing information for getting from idea to product


Using it in your own project.

1. cp ../idea-to-product/itp.xml .
2. ln -s ../idea-to-product/Makefile
3. ln -s ../idea-to-product/itp2doc.pl
4. ln -s ../idea-to-product/My


To validate the XML structure:
*  clear; rxp -V itp.xml > t


When adding a new component:
1. Create the component.
2. Add the relevant ModuleRelation
3. If the the component has children, then create a view packet for the component.
   * A view packet, for Module decomposition is only relevant if the if a sub-module in the view packet has sub-modules.
     Otherwise the list of sub-modules are in the primary presentation.


When implementing a new view packet style:
1. itp2doc.pl: Add the new entry to GenerateSoftwareArchitectureDocumentation().
   * e.g.   GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szAllocationTypeName, $f_szImplementationStyleName);
2. LaTeX_SwArcDoc.tmpl: Add code to retrieve the data (e.g. Type: Allocation Style: Implementation :
   * The array name is automatically generated in GenerateViewPacketByTypeAndStyle(): $refhSwArcHashList->{"ar${szViewPacketTypeName}${szViewPacketStyleName}ViewPackets"} = \@arViewPacketFileList;
{ foreach my $szModuleFileName (@arAllocationImplementationViewPackets) {
    $OUT .= "\\input{$szModuleFileName}\n";
  }
}


At
