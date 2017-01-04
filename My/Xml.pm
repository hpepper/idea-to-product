#****h* My/Xml.pm
#NAME
#  Xml.pm
#
# SYNOPSIS
#
#  This module contains support functions for the xml files.
#
#  Getting data out of it, and storing data in it.
#
#
# DESCRIPTION
#
#
#
#
# EXAMPLE
#
#  See t/Xml.t
#
#
# FILES
#
# ATTRIBUTES
#
# SEE ALSO
#
#
# NOTES
#
#
# CAVEATS
#
#  This script uses XML::LibXML.
#
# DIAGNOSTICS
#
#
# BUGS
#
#
# AUTHOR
#
#
# HISTORY
#
#****

package My::Xml;
use strict;
use vars qw(@ISA @EXPORT $VERSION);
use XML::LibXML;

use Carp;

use Exporter;

$VERSION = 1.1.0;
@ISA = ('Exporter');
@EXPORT = qw(
             &GetChildDataBySingleTagName
             &GetDataArrayByTagName
	     &GetDataArrayByTagAndAttribute
             &GetDataHashByTagName
             &GetDataHashByTagNameAndAttribute
             &GetFirstSubNodeValues
	     &GetNodeArrayByTagAndAttribute
	     &GetNodeArrayByTagName
             &GetSingleChildNodeByTagAndAttribute
             &GetSingleChildNodeByTagName
             &GetSingleChildNodeValueByTagAndAttribute
             &GetChildValueListByTagAndAttribute
             &ListAtrributeforTag
             &LoadXmlStructure
             &LoadXmlTree
             &SetActiveRoot
            );




# ----------------------------------------------------------------------------
#****f* Xml.pm/ListAtrributeforTag
# NAME
# ListAtrributeforTag
# FUNCTION
#  Get the list of Attribute with NAME on tag TAG.
#  E.g. get list of valid component for a release.
#
# INPUTS
#   * xmlNode -- XML Node.
#   * szTagName -- Tag name.
#   * szAttributeName -- Attribute name.
#
# OUTPUT
#
#  Nothing.
#
# RETURN VALUE
#  Array with all attributes by specified name.
#
# NOTES
#
# No error will be given if it cant resolve the parm2 or parm3.
#
# SOURCE
sub ListAtrributeforTag {
  my $xmlNode=$_[0];
  my $szTagName=$_[1];
  my $szAttributeName=$_[2];


  my @arValues;

  my @arEntries=$xmlNode->getElementsByTagName($szTagName);
  # die if no entries.

  # Go through the entries and get the attribute
  foreach my $pEntry (@arEntries) {
    my $szAttributeValue=$pEntry->getAttribute($szAttributeName);
    if ( defined($szAttributeValue) ) {
      push(@arValues, $szAttributeValue);
    } #endif right attr.
  } # end for each.
  
  return(@arValues);
} # listattributefortag.
#****



# ----------------------------------------------------------------------------
#****f* Xml.pm/LoadXmlStructure
# NAME
#   LoadXmlStructure
# FUNCTION
#  
#   Create the Parser, load the xml file, and return the root of the document. 
#
# INPUTS
#   * szFileName -- name of XML file.
#
# OUTPUT
#
# XML tree loaded.  
#
# RETURN VALUE
#  ($root, $tree)
#  pointer to root of document tree.
#  xml tree pointer.
#
# NOTES
#
#  It does not verify if the file exist, before attempting to read it.
#
#  It does no look for a version of the XML file. This should be implemented.
# 
#
# SOURCE
sub LoadXmlStructure {

  my $szFileName=$_[0];

# verify that filename exists.

  # what happens to the parser ? is it needed or can it be destroyed ?
  my $xml_parser = XML::LibXML->new();
  my $tree;
  my $root;
  if ( -f $szFileName ) {
    $tree  = $xml_parser->parse_file($szFileName);
  }
  if ( defined($tree) ) {
   $root = $tree->getDocumentElement;
  }

  return($root, $tree);
} # end loadxmlstructure.
#****







# ----------------------------------------------------------------------------
#****f* Xml.pm/LoadXmlTree
# NAME
#   LoadXmlTree
# FUNCTION
#  
#   Create the Parser, load the xml file, and return the root of the document. 
#
# INPUTS
#   * szFileName -- name of XML file.
#
# OUTPUT
#
# XML tree loaded.  
#
# RETURN VALUE
#  
#  pointer to root of document tree, if successfull.
#  return 'undef' if fail.
#
# NOTES
#
#  It does not verify if the file exist, before attempting to read it.
#
#  It does no look for a version of the XML file. This should be implemented.
# 
#
# SOURCE
sub LoadXmlTree {

  my $szFileName=$_[0];

# verify that filename exists.

  # what happens to the parser ? is it needed or can it be destroyed ?
  my $xml_parser = XML::LibXML->new();
  my $tree;
  my $root;
  if ( -f $szFileName ) {
    $tree  = $xml_parser->parse_file($szFileName);
  } else {
    die("!!! File does not exist: $szFileName");
  }
  if ( defined($tree) ) {
   $root = $tree->getDocumentElement;
  } else {
    print "LoadXmlTree(): didn't parse $szFileName\n";
  }
  return($root);
} # end loadxmltree.
#****



# ------------------------------------------------------------ #
# XML
=pod

=head2 SetActiveRoot

=over 4

=item Description

  Mostly used on rel_components.xml.
  Call LoadXmlTree($szFileName), find the entry with the correct parm2 name. Return the pointer to that element.
   If parm3 is non-empty, then the component name must match that as well as the version number. 
  (Return pointer to element with both correct version and component name).

=item Input

 parm1: name of XML file.

 parm2: Release version, like TE51, AP63 etc.

 parm3: optional component name. If is not used it MUST be "", otherwise this function will fail, without warning.

=item Output

  Nothing.

=item Value Returned

  pointer to element that fits parm2, parm3 requirement.

=item Cautions

  No error will be given if it cant resolve the parm2 or parm3.

=back

=cut

# ------------------------------------------------------------ #
sub SetActiveRoot {
# die if less that #2
  if ( $#_ < 2 ) {
    die "SetActiveRoot(): Too few params($#_), need at least filename, basetag, releasver [comp name] ", __FILE__, " line ", __LINE__, "\n";
  }
  my $szFileName=$_[0];
  my $szBaseTag=$_[1];
  my $szRelVer=$_[2];
  # this is an optional param, only used in the component xml file.
  my $szComponentName=$_[3];

  if ( ! defined($szComponentName) ) {
    $szComponentName="";
  }

  my $szReleaseRead="";
  my $szComponentNameRead="";

  my $pReturn=0;
  my $xmlRoot=LoadXmlTree($szFileName);

  # Get list of all base entries.
  my @arEntries=$xmlRoot->getElementsByTagName($szBaseTag);
# die if no entries.

  # Go through the entries and find the one with the correct version number.
  foreach my $pEntry (@arEntries) {
    $szReleaseRead=$pEntry->getAttribute('Name');
    # If the optional component name is given (non empty), the look for that as well.
    if ( $szComponentName ne "" ) {
      $szComponentNameRead=$pEntry->getAttribute('Component');
    }
    if ( ( $szRelVer eq $szReleaseRead ) && (  $szComponentName eq $szComponentNameRead) ) {
#print "entry found.";
      $pReturn=$pEntry;
    } # endif.
  } # end foreach.
  return($pReturn);
} # end setactiveroot.



# ----------------------------------------------------------------------------
#****f* xml2xml.pl/GetFirstSubNodeValues
# NAME
#   GetFirstSubNodeValues
# FUNCTION
#  
#  
#
# INPUTS
#   *  -- .
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
sub GetFirstSubNodeValues {
  my $xmlNode=$_[0];
  my $szTagName=$_[1];

  my @arAnswer;;

  my @arNodes = $xmlNode->getChildrenByTagName($szTagName);
  # print "GetFirstSubNodeValues($xmlNode, $szTagName): $#arNodes / @arNodes\n";
  if ($#arNodes == -1) {
    my $szNodeName=$xmlNode->nodeName();
    my @arChildNodes = $xmlNode->childNodes();
    foreach my $xmlChildNode (@arChildNodes) {
      my $szName=$xmlChildNode->nodeName();
      print "-$szName-\n";
    }
    confess "Node <$szNodeName> doesn't have <$szTagName> as child.\n";
  }
  foreach my $xmlTmpNode (@arNodes) {
    my $szAnswer=$xmlTmpNode->textContent();
    #  print "Content of $#arNodes $xmlTmpNode  <$szTagName>: $szAnswer\n"; 
    push(@arAnswer, $szAnswer);
  } # end foreach.

  return(@arAnswer);
} # end getfirstsubnodevalue
#****


# ----------------------------------------------------------------------------
#****f* Xml.pm/GetSingleChildNodeByTagAndAttribute
# NAME
# 
# FUNCTION
#  
#  
#
# INPUTS
#   * xmlNode -- XML Node from whence the search will start.
#   * szTagName -- Tagname to look for.
#   * szAttributeName -- Attribute name to look for.
#   * szAttributeContent -- Value in attribute to look for.
#
# OUTPUT
#   Nothing.
#
# RETURN VALUE
#  First node that fullfills the criteria of the search.
#
# NOTES
#  This should be refactored to be a subset of GetListOfChildNodesByTagAndAttribute
# 
#
# SOURCE
sub GetSingleChildNodeByTagAndAttribute {
  my $xmlNode=$_[0];
  my $szTagName=$_[1];
  my $szAttributeName=$_[2];
  my $szAttributeContent=$_[3];

  # print "   GetSingleChildNodeByTagAndAttribute()\n";
  my @arNodeList=GetNodeArrayByTagAndAttribute($xmlNode, $szTagName, $szAttributeName, $szAttributeContent);
  # print @arNodeList;

  # Get the first value.
  my $xmlReturnNode=shift @arNodeList;

  return($xmlReturnNode);
} # end Getsinglechildnodebytagandattribute
#***






# ----------------------------------------------------------------------------
#****f* Xml.pm/GetSingleChildNodeByTagName
# NAME
# 
# FUNCTION
#  
#  
#
# INPUTS
#   * xmlNode -- XML Node from whence the search will start.
#   * szTagName -- Tagname to look for.
#   * szAttributeName -- Attribute name to look for.
#
# OUTPUT
#   Nothing.
#
# RETURN VALUE
#  First node that fullfills the criteria of the search.
#
# NOTES
# 
#
# SOURCE
sub GetSingleChildNodeByTagName {
  my $xmlNode   = shift;
  my $szTagName = shift;

  # print "   GetSingleChildNodeByTagName()\n";
  my @arNodeList=GetNodeArrayByTagName($xmlNode, $szTagName);
  # print @arNodeList;

  # Get the first value.
  my $xmlReturnNode=shift @arNodeList;

  return($xmlReturnNode);
} # end GetSingleChildNodeByTagName
#***


# ----------------------------------------------------------------------------
# ----------------------------------------------------------------------------
sub GetSingleChildNodeValueByTagAndAttribute {
  my $xmlNode=$_[0];
  my $szTagName=$_[1];
  my $szAttributeName=$_[2];
  my $szAttributeContent=$_[3];

  my $szAnswer="";

  my $xmlFoundNode=GetSingleChildNodeByTagAndAttribute($xmlNode, $szTagName, $szAttributeName, $szAttributeContent);

  $szAnswer=$xmlFoundNode->string_value();

  return($szAnswer);

} #end getsinglechildnodevaluebytagandattribute


# ----------------------------------------------------------------------------
#****f* Xml.pm/GetDataArrayByTagAndAttribute
# NAME
#   GetDataArrayByTagAndAttribute
# FUNCTION
#  
#  
#
# INPUTS
#   * xmlNode -- XML Node from whence the search will start.
#   * szTagName -- Tagname to look for.
#   * szAttributeName -- Attribute name to look for.
#   * szAttributeContent -- Value in attribute to look for.
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# SEE ALSO
#   utGetDataArrayByTagAndAttribute.001 
#
# SOURCE
sub GetDataArrayByTagAndAttribute {
  if ( $#_ != 3 ) {
    die "GetDataArrayByTagAndAttribute(): wrong parameter number params($#_), need at least xmlNode and TagName AttributeName AttributeValue", __FILE__, " line ", __LINE__, "\n";
  }

  my $xmlNode=$_[0];
  my $szTagName=$_[1];
  my $szAttributeName=$_[2];
  my $szAttributeContent=$_[3];

  my @arDataList;

  my @arNodeList=GetNodeArrayByTagAndAttribute($xmlNode, $szTagName, $szAttributeName, $szAttributeContent);

  foreach my $xmlDataNode (@arNodeList) {
    my $szData=$xmlDataNode->string_value();
    push(@arDataList, $szData);
  } # end foreach

  return(@arDataList);
} # end getdataarraybytagandattribute.
#****



# ----------------------------------------------------------------------------
#****f* Xml.pm/GetNodeArrayByTagAndAttribute
# NAME
#   GetNodeArrayByTagAndAttribute
# FUNCTION
#  Get a list of nodes, with the given tag name.
#    If the szAttributeName is three spaces then only the szTagName is used.
#    
#  
#
# INPUTS
#   * xmlNode -- XML Node from whence the search will start.
#   * szTagName -- Tagname to look for.
#   * szAttributeName -- Attribute name to look for. use three spaces to disregard the attributes.
#   * szAttributeContent -- Value in attribute to look for.
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
sub GetNodeArrayByTagAndAttribute {
  if ( $#_ != 3 ) {
    die "GetNodeArrayByTagAndAttribute(): wrong parameter number params($#_), need at least xmlNode and TagName AttributeName AttributeValue", __FILE__, " line ", __LINE__, "\n";
  }

  my $xmlNode=$_[0];
  my $szTagName=$_[1];
  my $szAttributeName=$_[2];
  my $szAttributeContent=$_[3];

  my @arNodeList;

  if ( ( ! defined($xmlNode) ) || ( $xmlNode == 0 ) ) {
    confess "GetNodeArrayByTagAndAttribute(): zero pointer or undefined pointer for ($szTagName)\n";
  }

  my @arEntries=$xmlNode->getElementsByTagName($szTagName);
  # die if no entries.

  if ( $szAttributeName eq "   " ) {
    @arNodeList=@arEntries;
  } else {
    # Go through the entries and find any one with the attribute.
    if ( ! defined($szAttributeContent) ) {
      confess("szAttributeContent (parm3) not defined for szTagName($szTagName), szAttributeName($szAttributeName)");
    } else {
      foreach my $pEntry (@arEntries) {
        if ( $pEntry->hasAttribute($szAttributeName) ) {
          my $szAttributeRead=$pEntry->getAttribute($szAttributeName);
          if ( $szAttributeRead eq $szAttributeContent ) {
            push(@arNodeList, $pEntry);
          } #endif right attr.
        } # endif has attribute.
      } # end foreach.
    } # end else if szattributename not defined.
  } # end else.

  return(@arNodeList);
} # end getnodearraybytagandattribute
#****




sub GetNodeArrayByTagName {
  if ( $#_ != 1 ) {
    confess "GetNodeArrayByTagName(): wrong parameter number params($#_), need at least xmlNode and TagName\n";
  }
  my $xmlNode=$_[0];
  my $szTagName=$_[1];

  my @arNodeList=GetNodeArrayByTagAndAttribute($xmlNode, $szTagName, "   ", " ");

  return(@arNodeList);
}


# ----------------------------------------------------------------------------
#****f* Xml.pm/GetDataArrayByTagName
# NAME
#   GetDataArrayByTagName
# FUNCTION
#  Return an array with the data from all nodes identified by tagname.
#  
#
# INPUTS
#   * xmlNode -- XML Node from where to start search.
#   * xmlTagName -- Tag name to look for.
#
# OUTPUT
#
#  Nothing
#
# RETURN VALUE
#  Array with all data elements found.
#
# NOTES
#   This could be refactored to be a sub set or GetDataArrayByTagAndAttribute
# EXAMPLES
#   utGetDataArrayByTagName.001
# SOURCE
sub GetDataArrayByTagName {
  if ( $#_ != 1 ) {
    confess "GetDataArrayByTagName(): wrong parameter number params($#_), need at least xmlNode and TagName ", __FILE__, " line ", __LINE__, "\n";
  }

  my $xmlNode=$_[0];
  my $xmlTagName=$_[1];

  if ( ( ! defined($xmlNode) ) || ( $xmlNode == 0 ) ) {
    confess "GetChildDataBySingleTagName(): zero pointer or undefined pointer for ($xmlTagName) ", __FILE__, " line ", __LINE__, "\n";
  }


  my @arReturnList;
  
  # Read the Files list
  my @arElementList = $xmlNode->getElementsByTagName($xmlTagName);
  foreach my $szElement (@arElementList) {
    my $szData = $szElement->string_value();
    push(@arReturnList, $szData);
  } #end foreach.

  return(@arReturnList);
} # end getdataarraybytagname.
#****


# -----------------------------------------------------------------------
# ---------------------
sub GetArrayOfContentForEachElementByTagSortedByAttribute {
  my $xmlElement = shift;
  my $szTagName = shift;
  my $szSortAttributeName = shift;

  my @arResult;

  # TODO C Sort these entries by szSortAttributeName
  my @arNodeList = GetNodeArrayByTagName($xmlElement, $szTagName);
#  arNodeList.each { |xmlNode|
#    arResult.push(xmlNode.text)
#  }

  return(@arResult);
}

# ----------------------------------------------------------------------------
#****f* Xml.pm/GetDataHashByTagName
# NAME
#   GetDataHashByTagName
# FUNCTION
#  
#  Use the root, get the list sub tags
#   Read the value of each tag.
#
# INPUTS
#   * xmlNode -- xml node.
#   * xmlTagName -- tagname.
# 
#
# OUTPUT
#  Nothing.
#
# RETURN VALUE
# An array;
#   Each entry in the array is a hash of the values.
#   The data value of the node is stored in the hash under "   "
#    Since space can't be used as Attribute names.
#
# NOTES
#  
# EXAMPLE
#   See utGetDataHashByTagName.001
#
#   my @arAnswer=GetDataHashByTagName($xmlTreeRoot,"UrlLink");
#   foreach my $aohDate (@arAnswer) {
#     print FOOTER_MAS "     <a href=\"" . $aohDate->{"   "} . "\">" . $aohDate->{"Name"} . "</a> |\n";
#   } # endforeach.

# TODO
#  Store the DataKey in a global shared var.
# SOURCE
sub GetDataHashByTagName {
  # If there isn't two parameters barf.
  if ( $#_ != 1 ) {
    die "GetDataHashByTagName(): wrong parameter number params($#_), need at least xmlNode and TagName ", __FILE__, " line ", __LINE__, "\n";
  }

  my $xmlNode=shift;
  my $xmlTagName=shift;

  # Barf if the node isn't defined.
  if ( ( ! defined($xmlNode) ) || ( $xmlNode == 0 ) ) {
    die "GetChildDataBySingleTagName(): zero pointer or undefined pointer for ($xmlTagName) ", __FILE__, " line ", __LINE__, "\n";
  }

  my @arReturnList;

  
  # Get the list of subnodes identified by tagname.
  my @arNodeList = $xmlNode->getElementsByTagName($xmlTagName);

  # Go through each of the nodes found, extract the data from them.
  foreach my $szNode (@arNodeList) {
    # Initialize the hash for the data.
    my $hDataAndAttributes={};

    # Three spaces are the data key.
    $hDataAndAttributes->{"   "} = $szNode->string_value();

    # Get list of all attributes for the current node.
    my @arAttributeList=$szNode->attributes();

    # Itterate through the attribute, get the key and value.
    foreach my $szAttr (@arAttributeList) {
      # get the value.
      my $szValue=$szAttr->getValue();
      # get the name of attribute(key).
      my $szKey=$szAttr->nodeName();
      # Store key and value in the hash.
      $hDataAndAttributes->{$szKey} = $szNode->getAttribute($szKey);
    } # end foreach key.

    # Store the hash in the array.
    push(@arReturnList, $hDataAndAttributes);
  } #end foreach.
  # print "Result: "; print $arReturnList[1]{"Name"}; print "\n";

  return(@arReturnList);
} # end getdatahashbytagname
#****



# ----------------------------------------------------------------------------
#****f* Xml.pm/GetDataHashByTagNameAndAttribute
# NAME
#   GetDataHashByTagNameAndAttribute
# FUNCTION
#  
#  Use the root, get the list sub tags
#   Read the value of each tag.
#
# INPUTS
#   * xmlNode -- xml node.
#   * xmlTagName -- tagname.
#   * szAttributeName -- Attribute name to look for.
#   * szAttributeContent -- Value in attribute to look for.
# 
#
# OUTPUT
#  Nothing.
#
# RETURN VALUE
# An array;
#   Each entry in the array is a hash of the values.
#   The data value of the node is stored in the hash under "   "
#    Since space can't be used as Attribute names.
#
# NOTES
#  
# EXAMPLE
#   See utGetDataHashByTagNameAndAttribute.001
#
#   my @arAnswer=GetDataHashByTagNameAndTag($xmlTreeRoot,"UrlLink");
#   foreach my $aohDate (@arAnswer) {
#     print FOOTER_MAS "     <a href=\"" . $aohDate->{"   "} . "\">" . $aohDate->{"Name"} . "</a> |\n";
#   } # endforeach.

# TODO
#  Store the DataKey in a global shared var.
# SOURCE
sub GetDataHashByTagNameAndAttribute {
  # If there isn't two parameters barf.
  if ( $#_ != 3 ) {
    die "GetDataHashByTagNameAndAttribute(): wrong parameter number params($#_), need at least xmlNode and TagName ", __FILE__, " line ", __LINE__, "\n";
  }

  my $xmlNode=shift;
  my $xmlTagName=shift;
  my $szAttributeName=shift;
  my $szAttributeContent=shift;

  # Barf if the node isn't defined.
  if ( ( ! defined($xmlNode) ) || ( $xmlNode == 0 ) ) {
    die "GetDataHashByTagNameAndAttribute(): zero pointer or undefined pointer for ($xmlTagName) ", __FILE__, " line ", __LINE__, "\n";
  }

  my @arReturnList;

  
  # Get the list of subnodes identified by tagname.
  my @arNodeList = $xmlNode->getElementsByTagName($xmlTagName);

  # Go through each of the nodes found, extract the data from them.
  foreach my $szNode (@arNodeList) {
    if ( $szNode->hasAttribute($szAttributeName) ) {
      if ( $szNode->getAttribute($szAttributeName) eq $szAttributeContent ) {
        # Initialize the hash for the data.
        my $hDataAndAttributes={};

        # Three spaces are the data key.
        $hDataAndAttributes->{"   "} = $szNode->string_value();

        # Get list of all attributes for the current node.
        my @arAttributeList=$szNode->attributes();

        # Itterate through the attribute, get the key and value.
        foreach my $szAttr (@arAttributeList) {
          # get the value.
          my $szValue=$szAttr->getValue();
          # get the name of attribute(key).
          my $szKey=$szAttr->nodeName();
          # Store key and value in the hash.
          $hDataAndAttributes->{$szKey} = $szNode->getAttribute($szKey);
        } # end foreach key.

        # Store the hash in the array.
        push(@arReturnList, $hDataAndAttributes);
      } # endif attr correct content
    } # endif has attr.
  } #end foreach.
  # print "Result: "; print $arReturnList[1]{"Name"}; print "\n";

  return(@arReturnList);
} # end getdatahashbytagnameandattribute
#****



# ----------------------------------------------------------------------------
#****f* Xml.pm/GetChildDataBySingleTagName
# NAME
# 
# FUNCTION
#  
# only return the first value in a posible list of values.
# doesn't handle if the tagname does not exist.
#  
#
# INPUTS
#   * xmlNode -- XML Node.
#   * xmlTagName -- Tag Name.
#
# OUTPUT
#
#
# RETURN VALUE
#  
#  Text context of node with tag name.
#
# NOTES
#
# 
#
# SOURCE
sub GetChildDataBySingleTagName {
  my $xmlNode=$_[0];
  my $xmlTagName=$_[1];

  my $szAnswer;

  if ( ( ! defined($xmlNode) ) || ( $xmlNode == 0 ) ) {
    confess "GetChildDataBySingleTagName(): zero pointer or undefined pointer for ($xmlTagName) ", __FILE__, " line ", __LINE__, "\n";
  }

  # Read the Files list
  my @arElementList = $xmlNode->getElementsByTagName($xmlTagName);
#    confess "GetChildDataBySingleTagName(): returned empty childlist for \n";
  if ( $#arElementList > -1 ) {
    $szAnswer = $arElementList[0]->string_value();
  } # endif.
  return($szAnswer);
} # end getchilddatabytagname.
#****

1;
