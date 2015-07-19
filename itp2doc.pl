#!/usr/bin/perl -w


BEGIN {
 push(@INC, "$ENV{HOME}/local/perl");
 push(@INC, "$ENV{HOME}/lib");
}
use strict;
use Carp qw( confess longmess );
use Data::Dumper;
use FindBin;
use Text::Template;
use File::Basename;
use POSIX qw(strftime);

#use XML::Simple qw(:strict);

use Mot::Xml;


# ===========================================================================
#                          V A R I A B L E S
# ===========================================================================
my $f_szDefaultItpXmlFile = "itp.xml";
my $f_szAbsolutePathToCurrentScript = $FindBin::RealBin;
my $f_xmlRoot;

my $f_nViewPacketNumber;

my $f_nComponentRecursingLevelLeft = 0;

my $f_szSubMakefileName = "sub_makefile.mk";

# Holds the list of all targets being made.
my @f_arSubMakeTargetList;

# used as include file is the section could not be generated.
my $f_szNotPresent = "not_present.tex";

my $f_szComponentAndConnectorTypeName = "CnC";
my $f_szModuleTypeName = "Module";

# ============================================================
#                       F U N C T I O N S
# ============================================================


# -----------------------------------------------------------------
#  Take the input hash and generate a return hash where
#  the text, including keys have been converted to safe LaTeX text
#    e.g. '_' becomes '\_'
# See also: http://perldoc.perl.org/perlreftut.html
# ---------------
sub ConvertToLatexText {
# for getting key-value pairs of above hash
# foreach(keys%$hash) {
#   # by default key stored in $_ as per above foreach statement
#      # checks whether value of hash is another hash
#         if(ref($hash{$_}) eq 'HASH') {
#                 # iterate the keys set of inner hash
#                         foreach my $inner_key (keys%{$hash{$_}})    { 
#                                     # printing key and value of inner hash
#                                                 print "Key:$inner_key and value:$hash{$_}{$inner_key}\n"; 
#                                                         }
#                                                             } else {
#                                                                     print "Key: $_ and Value: $hash{$_}\n" 
#                                                                         }
#                                                                         }
}



# -----------------------------------------------------------------------
# ---------------------
sub GenerateContextModel {
  my $nId = shift;
  #puts "DDD  nId = (#{nId})"

  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ContextModel",  "Id", $nId);
  
  my %hContextModel;
 
  $hContextModel{'Id'} = $nId;
  $hContextModel{'Title'} = $xmlNode->getAttribute("Title");

  $hContextModel{'TheWorkId'} = GetChildDataBySingleTagName($xmlNode, "TheWorkEntityId");

  # TODO Read all entities, and sort them alphabetaically.
  my @arEntityList = GetNodeArrayByTagName($xmlNode, "Entity");

  my %hEntityNames;

  foreach my $xmlEntity (@arEntityList) {
    my $nEntityId = $xmlEntity->getAttribute("Id");
    $hEntityNames{$nEntityId} = { };
    $hEntityNames{$nEntityId}{"Name"} = $xmlEntity -> getAttribute("Name") ;
    # TODO C Fix this, I think it is wrong. The below stuff problably needs to be stored in the key of the Id.
    $hEntityNames{$nEntityId}{'Summary'} = GetChildDataBySingleTagName($xmlEntity, "Summary");
    $hEntityNames{$nEntityId}{'Description'} = GetChildDataBySingleTagName($xmlEntity, "Description");
  }
  $hContextModel{hEntityNames} = \%hEntityNames;

  # TODO read all relations and sort them alphabetically.
  # TODO Read all entities, and sort them alphabetaically.
  my @arRelation = GetNodeArrayByTagName($xmlNode, "Relation");

  my %hRelationNames;
  my $nCount = 0;

  foreach my $xmlRelation (@arRelation) {
    $hRelationNames{$nCount} = { "EntityAId" => $xmlRelation->getAttribute("EntityAId"),
                                 "EntityBId" => $xmlRelation->getAttribute("EntityBId"),
				 "Name"      => $xmlRelation->getAttribute("Name"),
                               };
      $hRelationNames{$nCount}{'Summary'} = GetChildDataBySingleTagName($xmlRelation, "Summary");
      $nCount += 1;
  } # end foreach xmlrelation.
  $hContextModel{hRelationNames} = \%hRelationNames;

    $hContextModel{GraphVizRankSeperation} = 1;
    if ( $#arEntityList < 5 ) {
      $hContextModel{GraphVizRankSeperation} = 1;
    } elsif ( $#arEntityList < 14 ) {
        $hContextModel{GraphVizRankSeperation} = 2;
    }   elsif ( $#arEntityList < 20 ) {
       $hContextModel{GraphVizRankSeperation} = 6;
    } elsif ( $#arEntityList < 35 ) {
         $hContextModel{GraphVizRankSeperation} = 7;
    }

  #puts "DDD #{$hEntityNames.length} => ranksep = #{$nGraphVizRankSeperation}"


  #print Dumper(\%hContextModel);

  # TODO C Populate the $hContextModel.

  # Generate a graph, using graphViz.
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/Graphviz_contextModel.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hContextModel);
  my $szNameWithoutSuffix = "ContextModel${nId}";
  my $szFileName = "${szNameWithoutSuffix}.gv";
  open( RESULT, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print RESULT $szResult;
  close(RESULT);

  $hContextModel{IncludeGraphicsName} = "${szNameWithoutSuffix}.ps";


  $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/LaTeX_contextModel.tmpl")
     or die "Couldn't construct template: $Text::Template::ERROR";



  $szResult = $template->fill_in(HASH => \%hContextModel);
  die("!!! LaTeX_contextModel.tmpl did not produce an output.") unless(defined($szResult));

  $szFileName = "${szNameWithoutSuffix}.tex";
  open( LATEX, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print LATEX $szResult;
  close(LATEX);

  UpdateSubMakefile("${szNameWithoutSuffix}.ps", "${szNameWithoutSuffix}.gv", "\ttwopi -Tps -o \$\@ \$^");
  UpdateSubMakefile("${szNameWithoutSuffix}.png", "${szNameWithoutSuffix}.gv", "\ttwopi -Tpng -o \$\@ \$^");

  return($szFileName);
}

# -----------------------------------------------------------------------
# ---------------------
sub GenerateFullCharter {
  my $nId = shift;

  my %hFullCharter = ReadFullCharterId($nId);
  #print Dumper(\%hFullCharter);
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/Wiki_fullCharter.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hFullCharter);
  my $szFileName = "charter_${nId}.wiki";
  open( RESULT, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print RESULT $szResult;
  close(RESULT);
}


# -----------------------------------------------------------------------
# ---------------------
sub GenerateInitialCharter {
  my $nId = shift;

  my %hInitialCharter = ReadInitialCharterId($nId);
  #print Dumper(\%hInitialCharter);
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/Wiki_initialCharter.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hInitialCharter);
  my $szFileName = "initialcharter_${nId}.wiki";
  open( RESULT, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print RESULT $szResult;
  close(RESULT);
}

# -----------------------------------------------------------------------
# Returns: szNameWithoutSuffix
# ---------------------
sub GenerateCncDiagramTree {
    my $nId = shift;
    my $szPacketType = shift;
    my $szPacketStyle = shift;
    my $szLevelLimit = shift || 1000;

    my $szModuleIncludeName;


     # TODO C Does the CnC need to be sorted on the Relation type? Does CnC have a relation type?.
     my %hComponentHashList = GenerateCncDiagramAndLegend($nId, "");
     #print Dumper(%hComponentHashList);
     $hComponentHashList{'GraphVizRankSeperation'} = 6; # TODO C Make this programmable.
    my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/Graphviz_${szPacketType}Diagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";

     my $szNameWithoutSuffix = "${szPacketType}DiagramTree${nId}";
     open(GRAPHVIZ, ">${szNameWithoutSuffix}.gv") or die("!!! Unable to create $szPacketType diagram file ${szNameWithoutSuffix}.gv: $!");
     my $szResult = $template->fill_in(HASH => \%hComponentHashList);
     print GRAPHVIZ "$szResult\n";
     close(GRAPHVIZ);

     $hComponentHashList{'IncludeGraphicsName'} = "${szNameWithoutSuffix}.ps";

     my %hTopComponent = GetComponentInformation($nId);
     $hComponentHashList{'GraphicsCaption'} = "Component diagram tree for " . $hTopComponent{'Name'};

     $hComponentHashList{'TableCaption'} = "Legend for component diagram tree - " . $hTopComponent{'Name'};


     $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_componentDiagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";
     $szResult = $template->fill_in(HASH => \%hComponentHashList);

     open(LATEX, ">${szNameWithoutSuffix}.tex") or die("!!! Unable to create $szPacketType diagram file: $!");
     print LATEX "$szResult\n";
     close(LATEX);

    my $szGraphVizCommand = "dot";

    # TODO V Should graphvic cmd be changed for C&C?
    # update Makefile
    # Target: Dependency  \n Command
    UpdateSubMakefile("${szNameWithoutSuffix}.ps", "${szNameWithoutSuffix}.gv", "\t$szGraphVizCommand -Tps -o \$\@ \$^");
    UpdateSubMakefile("${szNameWithoutSuffix}.png", "${szNameWithoutSuffix}.gv", "\t$szGraphVizCommand  -Tpng -o \$\@ \$^");

    return($szNameWithoutSuffix);
} # end GenerateCncDiagramTree

# -----------------------------------------------------------------
# ---------------
sub  GenerateCncDiagramAndLegend {
    my $nComponentId = shift;
    my $szRelationType = shift;
    my $szLevelLimit = shift || 1000;

    # The f_nComponentRecursingLevelLeft will be decremented by GenerateModuleComponentDiagramAndLegendRecursively().
    $f_nComponentRecursingLevelLeft = $szLevelLimit;

    # Arbitrary number to make it 'unlimited'.

    print "III GenerateCncDiagramAndLegend($nComponentId, $szRelationType)\n";


    GenerateCncDiagramAndLegendRecursively($nComponentId, $szRelationType,  $f_nComponentRecursingLevelLeft);
} # GenerateModuleComponentDiagramAndLegend

# -----------------------------------------------------------------
# ---------------
sub GenerateCncDiagramAndLegendRecursively {
   my $nComponentId = shift;
   my $szRelationType = shift;
   my $nComponentRecursingLevelLeft = shift;

#    $f_nComponentRecursingLevelLeft -= 1;
   print "III    GenerateModuleCncDiagramAndLegendRecursively($nComponentId, $szRelationType), ($nComponentRecursingLevelLeft)\n";

   my %hReturnComplexHashesList;
   $hReturnComplexHashesList{'arEntityList'} = ();
   $hReturnComplexHashesList{'arRelationList'} = ();

  if ( $nComponentRecursingLevelLeft  >= 0 ) {
  # TODO Extract info for the nComponentId here
    my %hComponentInformation = GetComponentInformation($nComponentId);
    $hComponentInformation{'Id'} = $nComponentId;
    push(@{$hReturnComplexHashesList{'arEntityList'}}, \%hComponentInformation);
    if ( $nComponentRecursingLevelLeft  > 0 ) {
      my @arSubComponentIdList = GetCncRelatedComponentList($nComponentId,  $szRelationType);
      foreach my $nSubComponentId (@arSubComponentIdList) {
        # Create the relation here, how do I make it (uses) generically?
        my %hRelation;
        $hRelation{ 'EntityAId'} = $nComponentId;
        $hRelation{ 'EntityBId'} = $nSubComponentId;
        $hRelation{ 'RelationType'}    = $szRelationType;
        $hRelation{ 'Name'}    = ""; # TODO V make this dependent on the relation type, or maybe make it empty.
        push(@{$hReturnComplexHashesList{'arRelationList'}}, \%hRelation);

        my %hTmpHash = GenerateCncDiagramAndLegendRecursively($nSubComponentId, $szRelationType, $nComponentRecursingLevelLeft-1);
        push(@{$hReturnComplexHashesList{'arEntityList'}}, @{$hTmpHash{'arEntityList'}});
        if ( exists($hTmpHash{'arRelationList'})  && defined($hTmpHash{'arRelationList'}) ) {
          push(@{$hReturnComplexHashesList{'arRelationList'}}, @{$hTmpHash{'arRelationList'}});
        }
      } # end foreach.
    } # endif >0
  } # endif.
  return(%hReturnComplexHashesList);
} # end GenerateCncDiagramAndLegendRecursively.




# -----------------------------------------------------------------
# ---------------
sub GenerateModuleComponentDiagramAndLegendRecursively {
   my $nComponentId = shift;
   my $szRelationType = shift;
   my $nComponentRecursingLevelLeft = shift;

#    $f_nComponentRecursingLevelLeft -= 1;
   print "III    GenerateModuleComponentDiagramAndLegendRecursively($nComponentId, $szRelationType), ($nComponentRecursingLevelLeft)\n";

   my %hReturnComplexHashesList;
   $hReturnComplexHashesList{'arEntityList'} = ();
   $hReturnComplexHashesList{'arRelationList'} = ();

  if ( $nComponentRecursingLevelLeft  >= 0 ) {
  # TODO Extract info for the nComponentId here
    my %hComponentInformation = GetComponentInformation($nComponentId);
    $hComponentInformation{'Id'} = $nComponentId;
    push(@{$hReturnComplexHashesList{'arEntityList'}}, \%hComponentInformation);
    if ( $nComponentRecursingLevelLeft  > 0 ) {
      my @arSubComponentIdList = GetSubComponentList($nComponentId,  $szRelationType);
      foreach my $nSubComponentId (@arSubComponentIdList) {
        # Create the relation here, how do I make it (uses) generically?
        my %hRelation;
        $hRelation{ 'EntityAId'} = $nComponentId;
        $hRelation{ 'EntityBId'} = $nSubComponentId;
        $hRelation{ 'RelationType'}    = $szRelationType;
        $hRelation{ 'Name'}    = ""; # TODO V make this dependent on the relation type, or maybe make it empty.
        push(@{$hReturnComplexHashesList{'arRelationList'}}, \%hRelation);

        my %hTmpHash = GenerateModuleComponentDiagramAndLegendRecursively($nSubComponentId, $szRelationType, $nComponentRecursingLevelLeft-1);
        push(@{$hReturnComplexHashesList{'arEntityList'}}, @{$hTmpHash{'arEntityList'}});
        if ( exists($hTmpHash{'arRelationList'})  && defined($hTmpHash{'arRelationList'}) ) {
          push(@{$hReturnComplexHashesList{'arRelationList'}}, @{$hTmpHash{'arRelationList'}});
        }
      } # end foreach.
      #print Dumper(@arSubComponentHashList);
      #print "DDD $#arSubComponentHashList\n";
    } # endif >0
  } # endif.
  #print "DDD done GenerateModuleComponentDiagramAndLegendRecursively($#arSubComponentHashList)\n";
  return(%hReturnComplexHashesList);
} # end GenerateModuleComponentDiagramAndLegendRecursively.


# -----------------------------------------------------------------
# ---------------
sub  GenerateModuleComponentDiagramAndLegend {
    my $nComponentId = shift;
    my $szRelationType = shift;
    my $szLevelLimit = shift || 1000;

    # The f_nComponentRecursingLevelLeft will be decremented by GenerateModuleComponentDiagramAndLegendRecursively().
    $f_nComponentRecursingLevelLeft = $szLevelLimit;

    # Arbitrary number to make it 'unlimited'.

    print "III GenerateModuleComponentDiagramAndLegend($nComponentId, $szRelationType)\n";


    GenerateModuleComponentDiagramAndLegendRecursively($nComponentId, $szRelationType,  $f_nComponentRecursingLevelLeft);
} # GenerateModuleComponentDiagramAndLegend



# -----------------------------------------------------------------------
# ---------------------
sub GenerateModuleDecompositionViewPacket {
 my $nId = shift;

  $f_nViewPacketNumber = 0;
  my $szContent = GenerateViewPacketRecusrsively($nId, "Module");

  print "------------------------------------------\n";

  my $szFileName = "moduleDecompositionViewPackets";
  open(FINISHED_DOC, ">${szFileName}.tex") or die("!!! Unable to open '${szFileName}.tex' for write: $!");
  print FINISHED_DOC $szContent;
  close(FINISHED_DOC);

}




# -----------------------------------------------------------------------
# TODO V possibly add 'AndLegend' to the end of the function name.
# Returns: szNameWithoutSuffix
# ---------------------
sub GeneratemoduleComponentDiagramTree {
    my $nId = shift;
    my $szLevelLimit = shift || 1000;

    my $szModuleIncludeName;

     my %hComponentHashList = GenerateModuleComponentDiagramAndLegend($nId, "is-part-of");
     #print Dumper(%hComponentHashList);
     $hComponentHashList{'GraphVizRankSeperation'} = 6; # TODO C Make this programmable.
     #print "DDD using template: ${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl\n";
    my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";

     my $szNameWithoutSuffix = "ComponentDiagramTree${nId}";
     open(GRAPHVIZ, ">${szNameWithoutSuffix}.gv") or die("!!! Unable to create module diagram file ${szNameWithoutSuffix}.gv: $!");
     my $szResult = $template->fill_in(HASH => \%hComponentHashList);
     print GRAPHVIZ "$szResult\n";
     close(GRAPHVIZ);

     $hComponentHashList{'IncludeGraphicsName'} = "${szNameWithoutSuffix}.ps";

     my %hTopComponent = GetComponentInformation($nId);
     $hComponentHashList{'GraphicsCaption'} = "Component diagram tree for " . $hTopComponent{'Name'};

     $hComponentHashList{'TableCaption'} = "Legend for omponent diagram tree - " . $hTopComponent{'Name'};


     $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_componentDiagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";
     $szResult = $template->fill_in(HASH => \%hComponentHashList);

     open(LATEX, ">${szNameWithoutSuffix}.tex") or die("!!! Unable to create module diagram file: $!");
     print LATEX "$szResult\n";
     close(LATEX);

    # update Makefile
    # Target: Dependency  \n Command
    UpdateSubMakefile("${szNameWithoutSuffix}.ps", "${szNameWithoutSuffix}.gv", "\tdot -Tps -o \$\@ \$^");
    UpdateSubMakefile("${szNameWithoutSuffix}.png", "${szNameWithoutSuffix}.gv", "\tdot -Tpng -o \$\@ \$^");

    return($szNameWithoutSuffix);
}



# -----------------------------------------------------------------------
# TODO C Put this as call in the Viewpacket gen thing.
# TODO C somehow coordinate the filename, possibly by returning it from here.
# ---------------------
sub GeneratemoduleComponentDiagram {
  my $nId = shift;

     my %hComponentHashList = GenerateModuleComponentDiagramAndLegend($nId, "is-part-of", 1);
#     print Dumper(%hComponentHashList);
     $hComponentHashList{'GraphVizRankSeperation'} = 6; # TODO C Make this programmable.
     print "DDD using template: ${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl\n";
#    print longmess("DDD GeneratemoduleComponentDiagram($nId) called.");
    confess("DDD GeneratemoduleComponentDiagram($nId) called.");
    my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";
     open(GRAPHVIZ, ">moduleComponentDiagram_${nId}.gv") or die("!!! Unable to create module diagram file: $!");
     my $szResult = $template->fill_in(HASH => \%hComponentHashList);
     print GRAPHVIZ "$szResult\n";
     close(GRAPHVIZ);
}



# -----------------------------------------------------------------------
# It must be possible to include the tex in other tex files.
# ---------------------
sub GenerateProjectExecutiveSummary {
  my $nId = shift;

  my %hProjectExecutiveCharter = ReadProjectExecutiveSummary($nId);

  $hProjectExecutiveCharter{'szDateGenerated'} = strftime("%Y-%m-%d", localtime());
  print Dumper(\%hProjectExecutiveCharter);
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/LaTeX_ProjectExecutiveSummary.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hProjectExecutiveCharter);
  my $szFileName = "ProjectExecutiveCharter_${nId}.tex";
  open( LATEX, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print LATEX $szResult;
  close(LATEX);
  
  my %hArticle;
  
  my $second_template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/LaTeX_article_project_executive_summary.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";
  $szResult = $second_template->fill_in(HASH => \%hArticle);
  $szFileName = "latex_project_executive_summary.tex";
  open( LATEX, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print LATEX $szResult;
  close(LATEX);
}


# -----------------------------------------------------------------------
# ---------------------
sub GetSipoc {
  my $nId = shift;
  my %hTemporary;

  print "DDD GetSipoc(${nId})\n";

  my $xmlElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Sipoc", "Id", $nId);
  if ( defined($xmlElement) ) {
    $hTemporary{"Headline"} = $xmlElement->getAttribute("Headline");
    push(@{$hTemporary{"Supplier"}}, GetDataArrayByTagName($xmlElement, "Supplier") );
    push(@{$hTemporary{"Input"}}, GetDataArrayByTagName($xmlElement, "Input") );
print "XXX TODO C Implement GetArrayOfContentForEachElementByTagSortedByAttribute() in the Xml.pm\n";
#    $hTemporary{"Step"} = GetArrayOfContentForEachElementByTagSortedByAttribute($xmlElement, "Step", "Order");
    push(@{$hTemporary{"Output"}}, GetDataArrayByTagName($xmlElement, "Output") );
    push(@{$hTemporary{"Customer"}}, GetDataArrayByTagName($xmlElement, "Customer") );
  }

  return(%hTemporary);
}

# -----------------------------------------------------------------------
# ---------------------
sub GetLatexSipoc {
  my $nId = shift;

  my %hSipocEntries = GetSipoc($nId);
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/LaTeX_sipoc.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hSipocEntries);
#  print Dumper(\%hSipocEntries);
#  die($szResult);
  return($szResult);
}

# -----------------------------------------------------------------
# ---------------
sub GetRequirementHashList {
  my @arRequimentIdList = @_;

  my %hBroadRequirements = ();
  foreach my $nReqId (@arRequimentIdList) {
    my $xmlRequirementNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Requirement", "Id", $nReqId);
    confess("EEE Cannot find a Requirement tag with Id = $nReqId") unless( defined($xmlRequirementNode) );
    my $szUuid = GetChildDataBySingleTagName($xmlRequirementNode, "uuid");
    $hBroadRequirements{$szUuid} = $xmlRequirementNode -> getAttribute("Headline");
  }
  return(%hBroadRequirements);
}

sub GenerateViewPacketByTypeAndStyle {
  my $xmlNode           = shift;
  my $refhSwArcHashList = shift;
  my $szType            = shift;
  my $szStyle           = shift;

  # TODO N Verify that the refhSwArcHashList is actually a hash reference.
 
  confess("EEE You must provide an XML node.") unless(defined($xmlNode));
  confess("EEE You must provide a packet Type and Style.") unless( defined($szType) && defined($szStyle) );

  my @arViewPacketFileList;
  my @arViewPacketList = GetDataArrayByTagName($xmlNode, "${szType}${szStyle}ViewPacketId");
  foreach my $nViewPacketId (@arViewPacketList) {
    $nViewPacketId += 0;
    if ( $nViewPacketId > 0 ) {
       my $szText = GenerateViewPacketRecusrsively($nViewPacketId, "Module");
        my $szViewPacketFileName = "Viewpacket${szType}${szStyle}${nViewPacketId}";
        push(@arViewPacketFileList, $szViewPacketFileName);
        open(LATEX, ">${szViewPacketFileName}.tex") || die("EEE unable to open file for write '${szViewPacketFileName}.tex': $!");
        print LATEX $szText;
        close(LATEX);
    }
  }

  $refhSwArcHashList->{"ar${szType}${szStyle}ViewPackets"} = \@arViewPacketFileList;

#  return(@arViewPacketFileList);
}


# -----------------------------------------------------------------
# ---------------
sub IncludeContextModelIfExists {
  my $xmlNode = shift;
  my $refhHash = shift;

  confess("EEE Missing parameters") unless (defined($refhHash));

  my $nContextModelId = GetChildDataBySingleTagName($xmlNode, "ContextModelId");
  
  if ( defined($nContextModelId) ) {
    $nContextModelId += 0;
  }  else {
    $nContextModelId = 0;
  }
  if ( $nContextModelId > 0 ) {
    my $szFileNameWithSuffix =  GenerateContextModel($nContextModelId);
    my($szFileName, $directories, $szSuffix) = fileparse($szFileNameWithSuffix, qr/\.[^.]*/);
    $refhHash->{'ContextModelFileName'} = $szFileName;
  } else {
    $refhHash->{'ContextModelFileName'} = $f_szNotPresent;
  }
}

# -----------------------------------------------------------------
# ---------------
sub GenerateSoftwareArchitectureDocumentation {
  my $nSwArcId = shift;

  my %hSwArcHashList;

  my %hProject;

  if ( -f $f_szSubMakefileName ) {
    # Remove the file if it exists.
    unlink($f_szSubMakefileName);
  }

  

  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "SoftwareArchitectureDocumentation",  "Id", $nSwArcId);
  die("!!! Node not found: SoftwareArchitectureDocumentation Id=$nSwArcId") unless(defined($xmlNode));

  $hProject{Title} = $xmlNode->getAttribute("Title");
  $hProject{'Author'} = GetChildDataBySingleTagName($xmlNode, "Author");
  $hProject{'ReleaseDate'} = GetChildDataBySingleTagName($xmlNode, "ReleaseDate");
  $hProject{'Issue'} = GetChildDataBySingleTagName($xmlNode, "Issue");
  $hProject{'Background'} = GetChildDataBySingleTagName($xmlNode, "Background");

  $hProject{'WhatWillNotBeDone'} = GetChildDataBySingleTagName($xmlNode, "WhatWillNotBeDone");
  $hProject{'PrototypeDeployment'} = GetChildDataBySingleTagName($xmlNode, "PrototypeDeployment");
  $hProject{'ProductionDeployment'} = GetChildDataBySingleTagName($xmlNode, "ProductionDeployment");
  $hProject{'PlannedUpgrades'} = GetChildDataBySingleTagName($xmlNode, "PlannedUpgrades");
  $hProject{'IntendedOperationalLife'} = GetChildDataBySingleTagName($xmlNode, "IntendedOperationalLife");

  IncludeContextModelIfExists($xmlNode, \%hProject);


  # TODO V Implement SIPOC section, when it is needed.
  my $nSipocId =  GetChildDataBySingleTagName($xmlNode, "SipocId");

  if (  $nSipocId  =~ /\d+/ ) {
    $nSipocId += 0;
  } else {
    $nSipocId = 0;
  }
  if ( $nSipocId > 0 ) {
    $hProject{"SystemLevelSipoc"} = GetLatexSipoc($nSipocId);
  } else {
    $hProject{"SystemLevelSipoc"} ="EMPTY";
  }

  my @arRequimentIdList = GetDataArrayByTagName($xmlNode, "RequirementId");
  my %hBroadRequirements = GetRequirementHashList(@arRequimentIdList);
  $hProject{'hBroadRequirements'} = \%hBroadRequirements;
  
  $hSwArcHashList{Project} = \%hProject;

  # TODO C Generate the views
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, "Module", "Decomposition");
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szComponentAndConnectorTypeName, "CommunicatingProcesses");



  print Dumper(\%hSwArcHashList);

  print "III Populating   LaTeX_SwArcDoc.tmpl\n";
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_SwArcDoc.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";
   my $szResult = $template->fill_in(HASH => \%hSwArcHashList);

  print "DDD Writing content to sad.tex.\n";
  open(LATEX, ">sad.tex") || die("!!! Unable to open file for writing: $!");
  print LATEX $szResult;
  close(LATEX);

  UpdateSubMakefile("all", join(" ", @f_arSubMakeTargetList), "");

} # end GenerateSoftwareArchitectureDocumentation.

# -----------------------------------------------------------------
# Return: The generated text.
# ---------------
sub GenerateViewPacketRecusrsively {
  my $nViewPacketId = shift;
  my $szDummy = shift;
  my $szComponentLevelLimit = shift || 1000;


  $f_nViewPacketNumber += 1;
  my %hModuleInformation;

  $hModuleInformation{ReferenceLabel} = "labelViewPacket$nViewPacketId";

  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $nViewPacketId);

  $hModuleInformation{PacketTitle} = $xmlNode->getAttribute("Title");
  #print "DDD $hModuleInformation{PacketTitle}\n";  
  my $szViewPacketType = GetChildDataBySingleTagName($xmlNode, "ViewPacketType");
  $hModuleInformation{ViewPacketType} = $szViewPacketType;
  my $szViewPacketStyle = GetChildDataBySingleTagName($xmlNode, "ViewPacketStyle");
  $hModuleInformation{ViewPacketStyle} = $szViewPacketStyle;
  $hModuleInformation{Introduction} = GetChildDataBySingleTagName($xmlNode, "Introduction");

  print "III GenerateViewPacketRecursively(${nViewPacketId}, ${szViewPacketType} - $szViewPacketStyle) ($f_nViewPacketNumber)\n";

  $hModuleInformation{PacketNumber} = $f_nViewPacketNumber;

  my $nPrimaryPresentationComponentId =   GetChildDataBySingleTagName($xmlNode, "ComponentId");
  if ( defined($nPrimaryPresentationComponentId) ) {
    # TODO V the 'module' part of the name should be derived from what view type and style is being generated.
    if ( $szViewPacketType eq $f_szModuleTypeName ) {
      $hModuleInformation{'PrimaryPresentationNameFile'}= GeneratemoduleComponentDiagramTree($nPrimaryPresentationComponentId, $szComponentLevelLimit);
    } elsif ( $szViewPacketType eq $f_szComponentAndConnectorTypeName ) {
      $hModuleInformation{'PrimaryPresentationNameFile'}= GenerateCncDiagramTree($nPrimaryPresentationComponentId, $szViewPacketType, $szViewPacketStyle, $szComponentLevelLimit);
    } else {
      confess("EEE View packet type not yet supported, please implement: $szViewPacketType");
    }
    # TODO C write gen command to the subcomponnent make file.
  } else {
    $hModuleInformation{'PrimaryPresentationNameFile'} = undef;
    confess("EEE No 'ComponentId' found in 'ViewPacket'  Id = $nViewPacketId");
  }

  IncludeContextModelIfExists($xmlNode, \%hModuleInformation);

#warn("XXX This is where we continue.");
# TODO XXX 
  # Modules use ModuleRelations
  # CnC use ComponentRelation
    #   <ConnectorAId>45</ConnectorAId>
    #   <ConnectorBId>17</ConnectorBId>

  

  # Get list of all 'ModuleRelations'
  # Filter on RelationType=is-part-of   and  ModuleBId=$nTopPacketId
  # TODO C The 'is-part-of' is dependent on the szViewType
  my @arSubComponentIdList = GetSubComponentList($nPrimaryPresentationComponentId, "is-part-of");

  my @arSubComponentHashList;
  foreach my $nComponentId (@arSubComponentIdList) {
      my %hComponentInformation = GetComponentInformation($nComponentId);
      push(@arSubComponentHashList, \%hComponentInformation);
  }
  $hModuleInformation{ComponentList} = \@arSubComponentHashList;
#  print Dumper(@arSubComponentHashList);
#  $hModuleInformation{TopModule} = \%{GetComponentInformation($nTopModuleId)};
  my %hTmp = GetComponentInformation($nPrimaryPresentationComponentId);
#  print Dumper(\%hTmp);
  # TODO V Do I still use this information?
  $hModuleInformation{TopModule} = \%hTmp;
#  print Dumper($hModuleInformation{TopModule});

  my @arParentViewPacketList = GetViewPacketLabelAndTitleHashList($nViewPacketId, "Parent" );
  $hModuleInformation{ParentViewPacketHashList} = \@arParentViewPacketList;
  my @arChildrenViewPacketList = GetViewPacketLabelAndTitleHashList($nViewPacketId, "Child" );
  $hModuleInformation{ChildViewPacketList} = \@arChildrenViewPacketList;
  my @arSiblingViewPacketList = GetViewPacketLabelAndTitleHashList($nViewPacketId, "Sibling" );
  $hModuleInformation{SiblingViewPacketList} = \@arSiblingViewPacketList;

  
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_${szViewPacketType}${szViewPacketStyle}ViewPacket.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hModuleInformation);
  if ( ! defined($szResult) ) {
    print Dumper(\%hModuleInformation);
  } 
  confess("EEE Template fill failed for LaTeX_${szViewPacketType}${szViewPacketStyle}ViewPacket.tmpl (template op: $?\n    Template error: '$Text::Template::ERROR')") unless(defined($szResult));

  my @arIdList =  GetDataArrayByTagName($xmlNode, "ChildViewPacketId");

  foreach my $nChildPacketViewId (@arIdList) {
      print "DDD nChildPacketViewId = $nChildPacketViewId\n";
      #print Dumper(\@arIdList);
      if (  $nChildPacketViewId  =~ /\d+/ ) {
         $nChildPacketViewId += 0;
        my $nComponentLevelsToShowInPrimaryPresentation = 1;
        my $szChildText = GenerateViewPacketRecusrsively($nChildPacketViewId, "DUMMY", $nComponentLevelsToShowInPrimaryPresentation);
        confess("EEE No text generated by: GenerateViewPacketRecusrsively($nChildPacketViewId, DUMMY, $nComponentLevelsToShowInPrimaryPresentation)") unless(defined($szChildText));
        $szResult .= $szChildText;
      }
  }
 

  return($szResult);
} # end GenerateViewPacketRecusrsively.


# -----------------------------------------------------------------
# ---------------
sub GetComponentInformation {
    my $nComponentId = shift;

    my %hComponentInformation;
    
    my $szTagNameToFind = "Component";

    my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, $szTagNameToFind,  "Id", "$nComponentId");
    confess("EEE Unable to get node for Tag: '$szTagNameToFind' and Id = $nComponentId") unless(defined($xmlNode));

    $hComponentInformation{Name} = $xmlNode->getAttribute("Name");
    $hComponentInformation{Summary} = GetChildDataBySingleTagName($xmlNode, "Summary");
    $hComponentInformation{Responsibilities} = GetChildDataBySingleTagName($xmlNode, "Responsibilities");
      
#    print Dumper(\%hComponentInformation);

    return(%hComponentInformation);
} #end GetComponentInformation

# -----------------------------------------------------------------
# ---------------
sub GetCncRelatedComponentList {
    my $nTopPacketId = shift;
    my $szRelationType = shift;

    my @arResponse;

    my $szRelationTagName = "ComponentRelation";

    if ( IsNumber($nTopPacketId) ) {
      my @arNodeList = GetNodeArrayByTagName($f_xmlRoot, $szRelationTagName);

      foreach my $xmlNode (@arNodeList) {
        my $szEntityAId = GetChildDataBySingleTagName($xmlNode, "ComponentAId");
        my $szEntityBId = GetChildDataBySingleTagName($xmlNode, "ComponentBId");
        my $szRelationType = GetChildDataBySingleTagName($xmlNode, "RelationType");
# TODO X I've swapped  szEntityBId and szEntityAId hoping this will help.
        if ( IsNumber($szEntityAId) ) {
          if ( ($szEntityAId == $nTopPacketId) && ( $szRelationType eq $szRelationType) ){
            push(@arResponse, $szEntityBId);
          } # endif
        } else {
	  my $nModuleRelationsId = $xmlNode->getAttribute("Id");
          warn("WWW ComponentAId was not a number: '$szEntityAId' for nTopPacketId=$nTopPacketId/$szRelationTagName Id: $nModuleRelationsId");
        }
      } # end foreach
    } else {
	confess("WWW nTopPacketId was not a number: '$nTopPacketId'");
    } # endif

    return(@arResponse);
} # end GetCncRelatedComponentList




# -----------------------------------------------------------------
# ---------------
sub GetSubComponentList {
    my $nTopPacketId = shift;
    my $szRelationType = shift;

    my @arResponse;

    if ( IsNumber($nTopPacketId) ) {
      my @arNodeList = GetNodeArrayByTagName($f_xmlRoot, "ModuleRelations");

      foreach my $xmlNode (@arNodeList) {
        my $szModuleAId =  GetChildDataBySingleTagName($xmlNode, "ModuleAId");
        my $szModuleBId = GetChildDataBySingleTagName($xmlNode, "ModuleBId");
        my $szRelationType = GetChildDataBySingleTagName($xmlNode, "RelationType");
        if ( IsNumber($szModuleBId) ) {
          if ( ($szModuleBId == $nTopPacketId) && ( $szRelationType eq $szRelationType) ){
            push(@arResponse, $szModuleAId);
          } # endif
        } else {
	  my $nModuleRelationsId = $xmlNode->getAttribute("Id");
          warn("WWW szModuleBId was not a number: '$szModuleBId' for nTopPacketId=$nTopPacketId/ModuleRations Id: $nModuleRelationsId");
        }
      } # end foreach
    } else {
	confess("WWW nTopPacketId was not a number: '$nTopPacketId'");
    } # endif

    return(@arResponse);
} # end GetSubComponentList



# -----------------------------------------------------------------
# ---------------
sub GetInherentSiblingList {
    my $nViewPacketId = shift;

    my @arSiblingList;

  # get the parent
  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $nViewPacketId);
    my $nParentId = GetChildDataBySingleTagName($xmlNode, "ParentViewPacketId");
    if (  !defined($nParentId) ) {
        # The top module has no parent.
        # So make sure that the ID is not an id.
	$nParentId = "X";
    }
  if ( $nParentId =~ /\d+/ ) {
        $nParentId += 0;
      my $xmlParentNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $nParentId);
     my @arIdList =  GetDataArrayByTagName($xmlParentNode, "ChildViewPacketId");
     foreach my $nId  (@arIdList) {
       if (  $nId  =~ /\d+/ ) {
        $nId += 0;
	if ( $nId != $nViewPacketId ) {
	    push(@arSiblingList, $nId);
        }
       } # endif.
      } # end foreach.
  }
  
  # get the children list
  # remove our own ID from the child list.

  # return the rest of the list.
    return(@arSiblingList);
} # end GetInherentSiblingList
  

# -----------------------------------------------------------------
# ---------------
sub GetViewPacketLabelAndTitleHashList {
  my $nViewPacketId = shift;
  my $szViewRelation = shift;

  print "III GetViewPacketLabelAndTitleHashList($nViewPacketId, $szViewRelation)\n";
  my @arLabelReferenceList;
   
  # Get the node for the viewpacket.
  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $nViewPacketId);

  my @arIdList =  GetDataArrayByTagName($xmlNode, "${szViewRelation}ViewPacketId");

  if ( $szViewRelation eq "Sibling" ) {
      my @arInherentSiblings = GetInherentSiblingList($nViewPacketId);
      push(@arIdList, @arInherentSiblings);
      print Dumper(@arIdList);
  }

  foreach my $szId (@arIdList) {
  # Label would be: 'labelViewPacket#'
    if (  $szId  =~ /\d+/ ) {
      $szId += 0;
      my $xmlTmpNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $szId);
      my $szTitle = $xmlTmpNode->getAttribute("Title");
      my %hViewPacketHash = ( "Label", "labelViewPacket$szId",
				"Title", $szTitle);
      push(@arLabelReferenceList, \%hViewPacketHash);
    }
  }

  return(@arLabelReferenceList);
} #end GetViewPacketLabel


# -----------------------------------------------------------------
# ---------------
sub IsNumber {
    my $szPossibleNumber = shift;

    my $nIsNumber = 0;
  if (  $szPossibleNumber  =~ /^\d+$/ ) {
      $nIsNumber = 1;
  }
    return($nIsNumber);
}

# -----------------------------------------------------------------------
# ======================================= To be moved to the ITP pm.
# ---------------------
sub ItpGetProjectType {
  my $nId = shift;

  my $szAnswer;

  my $xmlElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ProjectType",  "Id", $nId);
  confess("EEE unable to find ProjectType with ID = $nId") unless( defined($xmlElement) );
  $szAnswer = $xmlElement->getAttribute("Type");

  return($szAnswer);
}


# -----------------------------------------------------------------------
# ---------------------
sub ReadInitialCharterId {
  my $nId = shift;

  my %hInitialCharter;

  my $xmlInitialCharterElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "InitialCharter",  "Id", $nId);

  $hInitialCharter{"ProposalName"} = $xmlInitialCharterElement->getAttribute("ProposalName");

  my $xmlOpporttunityStatmentNode = GetSingleChildNodeByTagName($xmlInitialCharterElement, "OpporttunityStatment");
  $hInitialCharter{"CurrentState"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "CurrentState");
  $hInitialCharter{"WhatHasChanged"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "WhatHasChanged");
  $hInitialCharter{"WhatCouldTheConcequencesBe"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "WhatCouldTheConcequencesBe");
  $hInitialCharter{"ConsequencesIfNotChanged"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "ConsequencesIfNotChanged");
  $hInitialCharter{"DesiredState"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "DesiredState");
  $hInitialCharter{"When"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "When");
  $hInitialCharter{"GainAtDesiredState"} = GetChildDataBySingleTagName($xmlOpporttunityStatmentNode, "GainAtDesiredState");

  $hInitialCharter{"ExternalProductDependency"} = {};
  my @arList = GetNodeArrayByTagName($xmlInitialCharterElement, "ExternalProductDependency");
  foreach my $xmlExternalProductDependency (@arList) {
    $hInitialCharter{"ExternalProductDependency"}{ $xmlExternalProductDependency->getAttribute("ProductName") }
        = $xmlExternalProductDependency->getAttribute("Dependency");
  }


  my @arTmp = GetDataArrayByTagName($xmlInitialCharterElement, "TechnologyNeed");
  push(@{$hInitialCharter{"TechnologyDependencies"}}, @arTmp);

  @arTmp = GetDataArrayByTagName($xmlInitialCharterElement, "KnowledgeNeed");
  push(@{$hInitialCharter{"KnowledgeDependencies"}}, @arTmp);

  @arTmp = GetDataArrayByTagName($xmlInitialCharterElement, "KeyObjective");
  push(@{$hInitialCharter{"KeyObjectives"}}, @arTmp);

  $hInitialCharter{"Motive"}                 = GetChildDataBySingleTagName($xmlInitialCharterElement, "Motive");

  $hInitialCharter{"WhatIsBeingChangedOrProduced"}   = GetChildDataBySingleTagName($xmlInitialCharterElement, "WhatIsBeingChangedOrProduced");
  $hInitialCharter{"InitialMarketSegment"}  = GetChildDataBySingleTagName($xmlInitialCharterElement, "InitialMarketSegment");
  $hInitialCharter{"WhatIsBeingChangedOrProduced"} = GetChildDataBySingleTagName($xmlInitialCharterElement, "WhatIsBeingChangedOrProduced");

  my $xmlMissionStatmentNode = GetSingleChildNodeByTagName($xmlInitialCharterElement, "MissionStatment");

  $hInitialCharter{"ForTargetCustomers"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "ForTargetCustomers");
  $hInitialCharter{"WhoStatementOfNeedOrOpportunity"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "WhoStatementOfNeedOrOpportunity");
  $hInitialCharter{"TheProductName"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "TheProductName");
  $hInitialCharter{"IsAProductCategory"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "IsAProductCategory");
  $hInitialCharter{"ThatKeyBenefitCompellingReassonToBuyUse"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "ThatKeyBenefitCompellingReassonToBuyUse");
  $hInitialCharter{"UnlikePrimaryCompetitiveAltenativeOrCurrentSystemOrProcess"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "UnlikePrimaryCompetitiveAltenativeOrCurrentSystemOrProcess");
  $hInitialCharter{"OurProductPrimaryDiffAndAdvantageOfNewProduct"} = GetChildDataBySingleTagName($xmlMissionStatmentNode, "OurProductPrimaryDiffAndAdvantageOfNewProduct");

  return(%hInitialCharter);
}

# -----------------------------------------------------------------------
# ---------------------
sub ReadFullCharterId {
  my $nId = shift;

  my %hFullCharter;
  my $xmlFullCharterElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Charter",  "Id", $nId);
  my $nInitialCharterId = GetChildDataBySingleTagName($xmlFullCharterElement, "InitialCharterId");


  # TODO V test if the id is non nill.
  %hFullCharter = ReadInitialCharterId($nInitialCharterId);

  $hFullCharter{"CharterName"} = $xmlFullCharterElement->getAttribute("Name");

  $hFullCharter{"ProjectType"} = ItpGetProjectType( GetChildDataBySingleTagName($xmlFullCharterElement, "ProjectTypeId") );
  $hFullCharter{"ProjectOwner"} = GetChildDataBySingleTagName($xmlFullCharterElement, "ProjectOwner");

  $hFullCharter{"PitchFiveSeconds"} = GetChildDataBySingleTagName($xmlFullCharterElement, "PitchFiveSeconds");
  $hFullCharter{"PitchFiveMinutes"} = GetChildDataBySingleTagName($xmlFullCharterElement, "PitchFiveMinutes");
  $hFullCharter{"Product"} = GetChildDataBySingleTagName($xmlFullCharterElement, "ProductHeadline");

  my @arTmp = GetDataArrayByTagName($xmlFullCharterElement, "KeyFeature");
  push(@{$hFullCharter{"KeyFeatures"}}, @arTmp);


  # TODO V support a context model.
  my $xmlProjectScopeElement = GetSingleChildNodeByTagName($xmlFullCharterElement, "ProjectScope");
  @arTmp = GetDataArrayByTagName($xmlFullCharterElement, "InScope");
  push(@{$hFullCharter{"InScope"}}, @arTmp);

  @arTmp = GetDataArrayByTagName($xmlFullCharterElement, "OutOfScope");
  push(@{$hFullCharter{"OutOfScope"}}, @arTmp);

  $hFullCharter{"ImpactOnOtherBusinessUnits"} = GetChildDataBySingleTagName($xmlFullCharterElement, "ImpactOnOtherBusinessUnits");

  @arTmp = GetDataArrayByTagName($xmlFullCharterElement, "SupportRequired");
  push(@{$hFullCharter{"SupportRequired"}}, @arTmp);


  $hFullCharter{"ProjectObjectives"} = [];
  my @arNodeList = GetNodeArrayByTagName($xmlFullCharterElement, "ProjectObjectives");
  foreach my $xmlNode (@arNodeList) {
    my %hTemporary;
    $hTemporary{"Action"} = GetChildDataBySingleTagName($xmlNode, "Action");
    $hTemporary{"CardinalObjective"} = GetChildDataBySingleTagName($xmlNode, "CardinalObjective");
    $hTemporary{"ProjectSubject"} = GetChildDataBySingleTagName($xmlNode, "ProjectSubject");
    $hTemporary{"MarketSegment"} = GetChildDataBySingleTagName($xmlNode, "MarketSegment");
    $hTemporary{"Direction"} = GetChildDataBySingleTagName($xmlNode, "Direction");
    $hTemporary{"Measure"} = GetChildDataBySingleTagName($xmlNode, "Measure");
    $hTemporary{"Target"} = GetChildDataBySingleTagName($xmlNode, "Target");
    $hTemporary{"Deadline"} = GetChildDataBySingleTagName($xmlNode, "Deadline");
    #print Dumper(\%hTemporary);
    push(@{$hFullCharter{"ProjectObjectives"}}, \%hTemporary);
  }

  $hFullCharter{"RolledThroughputYieldGoal"} = GetChildDataBySingleTagName($xmlFullCharterElement, "RolledThroughputYieldGoal");
  $hFullCharter{"RolledThroughputYieldMaximum"} = GetChildDataBySingleTagName($xmlFullCharterElement, "RolledThroughputYieldMaximum");
#  # TODO N iterate over all 'RtyProcess' elements. -->
  my $xmlRtyProcessElement = GetSingleChildNodeByTagName($xmlFullCharterElement, "RtyProcess");
  my $fQtyIn     = GetChildDataBySingleTagName($xmlRtyProcessElement, "QuantityIn");
  $fQtyIn += 0.0;
  my $fQtyScrap  = GetChildDataBySingleTagName($xmlRtyProcessElement, "QuantityScrapped");
  $fQtyScrap += 0.0;
  my $fQtyRework = GetChildDataBySingleTagName($xmlRtyProcessElement, "QuantityReworked");
  $fQtyRework += 0.0;

  my $fRty = 0;
  if ( $fQtyIn != 0 ) {
    $fRty = 100 * ( ($fQtyIn - $fQtyScrap - $fQtyRework)/$fQtyIn );
  }
  #print "$fRty = 100 * ( ($fQtyIn - $fQtyScrap - $fQtyRework)/$fQtyIn );\n";
  $hFullCharter{"calculatedRty"} = sprintf("%.0f", $fRty);

  return(%hFullCharter);
}




# -----------------------------------------------------------------------
# ---------------------
sub ReadProjectStructureForGivenId {
  my $nId = shift;

  my %hProject;

  my $xmlProjectElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Project",  "Id", $nId);

  $hProject{"ProjectName"} = $xmlProjectElement->getAttribute("Name");
  $hProject{"CharterId"} = GetChildDataBySingleTagName($xmlProjectElement, "CharterId");
  $hProject{"ProductId"} = GetChildDataBySingleTagName($xmlProjectElement, "ProductId");
  $hProject{"ParentId"} = GetChildDataBySingleTagName($xmlProjectElement, "ParentId");
  $hProject{"ProjectTypeId"} = GetChildDataBySingleTagName($xmlProjectElement, "ProjectTypeId");
  $hProject{"ProjectDescription"} = GetChildDataBySingleTagName($xmlProjectElement, "Description");
  $hProject{"TechLead"} = GetChildDataBySingleTagName($xmlProjectElement, "TechLead");

  my @arRequimentIdList = GetDataArrayByTagName($xmlProjectElement, "RequirementId");
  my %hBroadRequirements = GetRequirementHashList(@arRequimentIdList);
  $hProject{'hBroadRequirements'} = \%hBroadRequirements;

  my @arProjectRisks;

  my @arRiskIdList = GetDataArrayByTagName($xmlProjectElement, "RiskId");
  foreach my $szRiskId (@arRiskIdList) {
    my %hRisk;
    $hRisk{'Id'} = $szRiskId;
    my $xmlRiskNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Risk",  "Id", $szRiskId);
    $hRisk{'Name'} = $xmlRiskNode->getAttribute("Name");
    $hRisk{Description} = GetChildDataBySingleTagName($xmlRiskNode, "Description");
    push(@arProjectRisks, \%hRisk);
  }

  $hProject{"ProjectRisks"} = \@arProjectRisks;

  my @arProjectIssues;

  my @arIssueIdList = GetDataArrayByTagName($xmlProjectElement, "IssueId");
  foreach my $szIssueId (@arIssueIdList) {
    my %hIssue;
    $hIssue{'Id'} = $szIssueId;
    my $xmlIssueNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Issue",  "Id", $szIssueId);
    $hIssue{'Headline'} = $xmlIssueNode->getAttribute("Headline");
    $hIssue{Description} = GetChildDataBySingleTagName($xmlIssueNode, "Description");
    push(@arProjectIssues, \%hIssue);
  }

  $hProject{"ProjectIssues"} = \@arProjectIssues;

  my @arProjectStoryCards;
  my @arStoryCardIdList = GetDataArrayByTagName($xmlProjectElement, "MilestoneStoryCardId");
  foreach my $szStoryCardId (@arStoryCardIdList) {
    my %hStoryCard;
    $hStoryCard{'Id'} = $szStoryCardId;
    my $xmlStoryCardNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "StoryCard",  "Id", $szStoryCardId);
    $hStoryCard{'Headline'} = $xmlStoryCardNode->getAttribute("Headline");
    $hStoryCard{Description} = GetChildDataBySingleTagName($xmlStoryCardNode, "Description");
    $hStoryCard{Complexity} = GetChildDataBySingleTagName($xmlStoryCardNode, "Complexity");
    push(@arProjectStoryCards, \%hStoryCard);
  }
  $hProject{"ProjectStoryCards"} = \@arProjectStoryCards;


  #print Dumper(\%hProject);
  return(%hProject);
}


# -----------------------------------------------------------------------
# ---------------------
sub ReadProjectExecutiveSummary {
  my $nId = shift;

  my %hProjectExecutiveSummary;
  my $xmlProjectExecutiveSummaryElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ExecutiveProjectSummary",  "Id", $nId);
  
  # Read the Project structure
  my $nProjectId = GetChildDataBySingleTagName($xmlProjectExecutiveSummaryElement, "ProjectId");
  my %hProject = ReadProjectStructureForGivenId($nProjectId);
  print Dumper(\%hProject);
  
  # TODO V test if the id is non nill.
  %hProjectExecutiveSummary = ReadFullCharterId($hProject{'CharterId'});

  foreach my $szKey (keys %hProject) {
    $hProjectExecutiveSummary{$szKey} = $hProject{$szKey};
  }
  

#  $hFullCharter{"CharterName"} = $xmlFullCharterElement->getAttribute("Name");

#  $hFullCharter{"ProjectType"} = ItpGetProjectType( GetChildDataBySingleTagName($xmlFullCharterElement, "ProjectTypeId") );


  return(%hProjectExecutiveSummary);
}

# -----------------------------------------------------------------------
# ---------------------
sub UpdateSubMakefile {
  my $szTarget = shift;
  my $szDependencies = shift;
  my $szCommands = shift;

  push(@f_arSubMakeTargetList, $szTarget);

  open(SUBMAKEFILE, ">>$f_szSubMakefileName") || die("EEE unable to open file for write '$f_szSubMakefileName': $!");
  print SUBMAKEFILE "$szTarget: $szDependencies\n";
  print SUBMAKEFILE "$szCommands\n";
  print SUBMAKEFILE "\n";
  close(SUBMAKEFILE);
}

# ============================================================

                #         #     #      ###   #       #
                ##   ##   # #       #      ##    #
                # # # #  #   #      #      # #   #
                #  #    # #     #     #      #  #  #
                #         # ####    #      #   # #
                #         # #      #    #      #    ##
                #         # #      #  ###  #       #

# ============================================================
print "DDD Begin\n";

if ( $#ARGV != 1 ) {
  die("!!! You must provide two options, a Section name and an Id");
}

$f_xmlRoot = LoadXmlTree($f_szDefaultItpXmlFile);
#$f_xmlRoot = XMLin($f_szDefaultItpXmlFile);
if ( ! defined($f_xmlRoot) ) {
    die("!!! XML root is undefined.");
} 

my $szSectionName = shift @ARGV;
my $nSectionId    = shift @ARGV;

if ( $szSectionName eq "SoftwareArchitectureDocumentation" ) {
  GenerateSoftwareArchitectureDocumentation($nSectionId);
} elsif ( $szSectionName eq "ContextModel" ) {
  GenerateContextModel($nSectionId);
} elsif ( $szSectionName eq "ModuleDecompositionViewPacket" ) {
  GenerateModuleDecompositionViewPacket($nSectionId);
} elsif ( $szSectionName eq "moduleComponentDiagram"  ) {
  GeneratemoduleComponentDiagram($nSectionId);
} elsif ( $szSectionName eq "moduleComponentDiagramTree"  ) {
  GeneratemoduleComponentDiagramTree($nSectionId); 
} elsif ( $szSectionName eq "Charter"  ) {
  GenerateFullCharter($nSectionId); 
} elsif ( $szSectionName eq "InitialCharter"  ) {
  GenerateInitialCharter($nSectionId); 
} elsif ( $szSectionName eq "ProjectExecutiveSummary"  ) {
  GenerateProjectExecutiveSummary($nSectionId);
} elsif ( $szSectionName eq "ViewPacket"  ) {
  GenerateViewPacketRecusrsively($nSectionId);
} else {
  die("!!! Unrecognized Section name: $szSectionName");
}


print "DDD End - last line\n";
