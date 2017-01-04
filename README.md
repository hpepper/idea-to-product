idea-to-product
===============

Tools for capturing information for getting from idea to product


Using it in your own project.

cp ../idea-to-product/itp.xml .
ln -s ../idea-to-product/Makefile
ln -s ../idea-to-product/itp2doc.pl
ln -s ../idea-to-product/My


To validate the XML structure:
  clear; rxp -V itp.xml > t


When adding a new component:
# Create the component.
# Add the relevant ModuleRelation
# If the the component has children, then create a view packet for the component.
#* A view packet, for Module decomposition is only relevant if the if a sub-module in the view packet has sub-modules.
     Otherwise the list of sub-modules are in the primary presentation.


When implementing a new view packet style:
# itp2doc.pl: Add the new entry to GenerateSoftwareArchitectureDocumentation().
#* e.g.   GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szAllocationTypeName, $f_szImplementationStyleName);
# LaTeX_SwArcDoc.tmpl: Add code to retrieve the data (e.g. Type: Allocation Style: Implementation :
#* The array name is automatically generated in GenerateViewPacketByTypeAndStyle(): $refhSwArcHashList->{"ar${szViewPacketTypeName}${szViewPacketStyleName}ViewPackets"} = \@arViewPacketFileList;
{ foreach my $szModuleFileName (@arAllocationImplementationViewPackets) {
    $OUT .= "\\input{$szModuleFileName}\n";
  }
}


At
