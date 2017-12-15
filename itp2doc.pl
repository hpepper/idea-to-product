#!/usr/bin/perl -w
use FindBin;

BEGIN {
 push(@INC, "$ENV{HOME}/local/perl");
 push(@INC, "$ENV{HOME}/lib");
 push(@INC, "$FindBin::RealBin");
}
use strict;
use Carp qw( confess longmess shortmess );
use Data::Dumper;
use Text::Template;
use File::Basename;
use POSIX qw(strftime);

#use XML::Simple qw(:strict);

use My::Xml;


# ===========================================================================
#                          V A R I A B L E S
# ===========================================================================
my $f_szVersion = "1.2.2";

my $f_szDefaultItpXmlFile = "itp.xml";
my $f_szAbsolutePathToCurrentScript = $FindBin::RealBin;
#"../idea-to-product";
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
my $f_szAllocationTypeName = "Allocation";
my $f_szSubTypeName = "Sub";

# Modul styles
my $f_szDecompositionStyleName = "Decomposition"; # was 'is-part-of'
my $f_szUsesStyleName = "Uses"; #
my $f_szGeneralizationStyleName = "Generalization"; #
my $f_szLayeredStyleName = "Layered"; #

# CnC styles
my $f_szCommunicatingProcessesStyleName = "CommunicatingProcesses"; #
my $f_szClientServerStyleName = "ClientServer"; #
my $f_szConcurrencyStyleName = "Concurrency"; #
my $f_szMasterSlaveStyleName = "MasterSlave"; #
my $f_szPeerToPeerStyleName = "PeerToPeer"; #
my $f_szPipeAndFilterStyleName = "PipeAndFilter"; #
my $f_szProduceConsumeStyleName = "ProduceConsume"; #
my $f_szPublishSubscribeStyleName = "PublishSubscribe"; #
my $f_szSharedDataStyleName = "SharedData"; #

# Allocation styles
my $f_szDeploymentStyleName = "Deployment"; #
my $f_szImplementationStyleName = "Implementation"; #
my $f_szWorkAssignmentStyleName = "WorkAssignment"; #

# Sub style
my $f_szContextModel = "ContextModel";

# ============================================================
#                       F U N C T I O N S
# ============================================================


# TODO C Change this to a handle.
open(LOG,">itp2doc.log") || die("!!! Unable to open file for write: itp2doc.log - $!");

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
#   # checks whether value of hash is another hash
#   if(ref($hash{$_}) eq 'HASH') {
#     # iterate the keys set of inner hash
#     foreach my $inner_key (keys%{$hash{$_}})    {
#       # printing key and value of inner hash
#       print "Key:$inner_key and value:$hash{$_}{$inner_key}\n";
#     }
#   } else {
#     print "Key: $_ and Value: $hash{$_}\n"
#   }
# }
}

# -----------------------------------------------------------------------
# ---------------------
sub Tee {
  my $szText = shift || confess("!!! Please provide a string with content.");

  print "$szText";
  Log($szText);
}


# -----------------------------------------------------------------------
# ---------------------
sub Log {
  my $szText = shift;

  print LOG "$szText";
}

# -----------------------------------------------------------------------
# ---------------------
sub GenerateContextModel {
    my $nViewPacketId = shift || confess("!!! You must provide a view packet ID as the first parameter");

    my %hViewPacketInformation = GetViewPacketInformation($nViewPacketId);

    my $nComponentId;
    if ( IsNumber($hViewPacketInformation{'ComponentId'}) ) {
    	$nComponentId = $hViewPacketInformation{'ComponentId'};
    } else {
    	Tee(Dumper(\%hViewPacketInformation));
    	confess("!!! Please provide a ComponentId in ViewPacket Id=$nViewPacketId");
    }

    my %hComponentHashList = GenerateComponentDiagramAndLegendRecursively($nComponentId, $f_szSubTypeName, $f_szContextModel,  1);

    $hComponentHashList{'TheWorkId'} = $nComponentId;

    $hComponentHashList{'GraphicsCaption'} = $hViewPacketInformation{'Title'};
    $hComponentHashList{'TableCaption'} = "Legend for $hViewPacketInformation{'Title'}";
    $hComponentHashList{'szViewPacketTypeName'} = $hViewPacketInformation{'ViewPacketType'};
    $hComponentHashList{'szViewPacketStyleName'} = $hViewPacketInformation{'ViewPacketStyle'};


    # TODO Read all entities, and sort them alphabetaically.
    # TODO read all relations and sort them alphabetically.

    my $nNumberOfEntities = scalar @{ $hComponentHashList{'arEntityList'} };

    $hComponentHashList{'GraphVizRankSeperation'} = 1;
    if ( $nNumberOfEntities < 5 ) {
      $hComponentHashList{'GraphVizRankSeperation'} = 1;
    } elsif ( $nNumberOfEntities < 14 ) {
        $hComponentHashList{'GraphVizRankSeperation'} = 2;
    }   elsif ( $nNumberOfEntities < 20 ) {
       $hComponentHashList{'GraphVizRankSeperation'} = 6;
    } elsif ( $nNumberOfEntities < 35 ) {
         $hComponentHashList{'GraphVizRankSeperation'} = 7;
    }

  #puts "DDD #{$hEntityNames.length} => ranksep = #{$nGraphVizRankSeperation}"

  #print Dumper(\%hContextModel);

  # Populate the $hContextModel.
  # Generate a graph, using graphViz.
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/Graphviz_contextModel.tmpl")
          or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hComponentHashList);
  if ( ! defined($szResult) ) {
  	Tee(Dumper(\%hComponentHashList));
  	confess("!!! Graphviz_contextModel.tmpl did not produce an output. $Text::Template::ERROR");
  }

  my $szNameWithoutSuffix = "ContextModel${nViewPacketId}";
  my $szFileName = "${szNameWithoutSuffix}.gv";
  open( RESULT, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print RESULT $szResult;
  close(RESULT);

  $hComponentHashList{'IncludeGraphicsName'} = "${szNameWithoutSuffix}.ps";


  $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/LaTeX_componentDiagram.tmpl")
     or die "Couldn't construct template: $Text::Template::ERROR";



  $szResult = $template->fill_in(HASH => \%hComponentHashList);
  die("!!! LaTeX_componentDiagram.tmpl did not produce an output.") unless(defined($szResult));

  $szFileName = "${szNameWithoutSuffix}.tex";
  open( LATEX, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print LATEX $szResult;
  close(LATEX);

  UpdateSubMakefile("${szNameWithoutSuffix}.ps", "${szNameWithoutSuffix}.gv", "\ttwopi -Tps -o \$\@ \$^");
  UpdateSubMakefile("${szNameWithoutSuffix}.png", "${szNameWithoutSuffix}.gv", "\ttwopi -Tpng -o \$\@ \$^");

  return($szFileName);
}

# -----------------------------------------------------------------------
# Generate UML diagrams, like: Sequence Diagram, etc.
# Called by IncludeDiagramIfExists
#
# @param DiagramId - ID for the diagram to be generated
#
#  Get list of actions, order the actions, then start generating the plantumlt code
#    TODO V How to handle 'return' flow?
# ---------------------
sub GenerateUmlDiagram {
    my $nDiagramId = shift || confess("!!! You must provide a diagram ID as the first parameter");

    if ( ! IsNumber($nDiagramId) ){
    	confess("!!! Please provide a valid DiagramId $nDiagramId");
    }

    # TODO C Load the  DiagramStructure (So title, description and Action list can be accessed.)
    # Returns The Action list as an array of hashes.
    my %hDiagramInformation = GetDiagramInformation($nDiagramId);

    my %hComponentHashList; # = GenerateComponentDiagramAndLegendRecursively($nComponentId, $f_szSubTypeName, $f_szContextModel,  1);

    $hComponentHashList{'GraphicsCaption'} = $hDiagramInformation{'Title'};
    $hComponentHashList{'TableCaption'} = "Legend for $hDiagramInformation{'Title'}";
    $hComponentHashList{'szViewPacketTypeName'} = $hDiagramInformation{'Type'};


    # read all relations and sort them alphabetically.
    my @arActionOrderNumberList;
    my %hActionList;
    # TODO N Report if two actions has the same order number.
    foreach my $refhAction ( @{$hDiagramInformation{'ActionList'}}) {
    	print "$refhAction->{Order}\n";
    	push(@arActionOrderNumberList, $refhAction->{Order});
    	$hActionList{$refhAction->{Order}} = $refhAction->{ComponentRelationId};
    }

    my @arSortedActionOrderNumberList = sort(@arActionOrderNumberList);


  my $szNameWithoutSuffix = "UmlDiagram${nDiagramId}";
  my $szFileName = "${szNameWithoutSuffix}.apt";
  open( RESULT, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print RESULT "\@startuml\n";
  print RESULT "\n";
  # BTS1 -> ZC1: req(1)
  # Loop through the sorted list of actions.
  foreach my $nActionOrder (@arSortedActionOrderNumberList) {
  	my %hComponentRelation =  mdlGetComponentRelationInformation($hActionList{$nActionOrder});
  	my $szComponentAName = mdlGetComponentNameFromId($hComponentRelation{'ComponentAId'});
  	my $szComponentBName = mdlGetComponentNameFromId($hComponentRelation{'ComponentBId'});
  	my $szConnectionTitle = $hComponentRelation{'ConnectionTitle'};

  	print RESULT "\"$szComponentAName\" -> \"$szComponentBName\": $szConnectionTitle\n";
  }
  if (defined($hDiagramInformation{'Verbatim'})) {
    print RESULT "$hDiagramInformation{'Verbatim'}\n";
  }
  print RESULT "\@enduml\n";
  close(RESULT);

  $hComponentHashList{'IncludeGraphicsName'} = "${szNameWithoutSuffix}.eps";


  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "$f_szAbsolutePathToCurrentScript/LaTeX_componentDiagram.tmpl")
     or die "Couldn't construct template: $Text::Template::ERROR";



  my $szResult = $template->fill_in(HASH => \%hComponentHashList);
  die("!!! LaTeX_componentDiagram.tmpl did not produce an output.") unless(defined($szResult));

  $szFileName = "${szNameWithoutSuffix}.tex";
  open( LATEX, ">$szFileName") || die("!!! Unable to open $szFileName for write: $!");
  print LATEX "$szResult";
  close(LATEX);

  # $@: Name of the current task
  # $^:
  UpdateSubMakefile("${szNameWithoutSuffix}.eps", "${szNameWithoutSuffix}.apt", "\tjava -jar plantuml.jar -teps \$^");
  UpdateSubMakefile("${szNameWithoutSuffix}.png", "${szNameWithoutSuffix}.apt", "\tjava -jar plantuml.jar -tpng \$^");

  return($szFileName);
}


# -----------------------------------------------------------------------
# Reads the Diagram information for the given DiagramId
#
# @param nDiagramId - Id of diagram to retrieve.
# @return Hash with the following informatin:
#          - Id
#          - Title
#          - Description
#          - Type
#          - Array of Hashes with all the actions.
# ---------------------
sub GetDiagramInformation {
	my $nDiagramId = shift || confess("!!! You must provide a diagram ID as the first parameter");

	my %hDiagramInformation;

    my $szTagNameToFind = "Diagram";

    Tee("III                GetDiagramInformation($nDiagramId)\n");


    my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, $szTagNameToFind,  "Id", "$nDiagramId");
    confess("EEE Unable to get node for Tag: '$szTagNameToFind' and Id = $nDiagramId") unless(defined($xmlNode));

    $hDiagramInformation{'Id'} = $nDiagramId;
    $hDiagramInformation{'Title'} = $xmlNode->getAttribute("Title");
    $hDiagramInformation{'Type'} = $xmlNode->getAttribute("Type");
    $hDiagramInformation{'Description'} = GetChildDataBySingleTagName($xmlNode, "Description");
    $hDiagramInformation{'Verbatim'} = GetChildDataBySingleTagName($xmlNode, "Verbatim");
    my @xmlNodes = GetNodeArrayByTagName($xmlNode, "Action");

    my @arActionList;
    foreach my $xmlActionNode (@xmlNodes) {
    	my %hAction;
    	$hAction{"Order"} = $xmlActionNode->getAttribute("Order");
    	$hAction{"ComponentRelationId"} = $xmlActionNode->getAttribute("ComponentRelationId");
    	push(@arActionList, \%hAction);
    }
    $hDiagramInformation{'ActionList'} = \@arActionList;


    return(%hDiagramInformation);
}


# -----------------------------------------------------------------------
# Reads the Diagram information for the given DiagramId
#
# @param nDiagramId - Id of diagram to retrieve.
# @return Hash with the following informatin:
#          - Id
#          - Title
#          - Description
#          - Type
#          - Array of Hashes with all the actions.
# ---------------------
sub mdlGetComponentRelationInformation {
	my $nId = shift || confess("!!! You must provide a Component Relation ID as the first parameter");

	my %hInformation;

    my $szTagNameToFind = "ComponentRelation";

    Tee("III                mdlGetComponentRelationInformation($nId)\n");


    my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, $szTagNameToFind,  "Id", "$nId");
    confess("EEE Unable to get node for Tag: '$szTagNameToFind' and Id = $nId") unless(defined($xmlNode));

    $hInformation{'Id'} = $nId;
    $hInformation{'ComponentAId'} = GetChildDataBySingleTagName($xmlNode, "ComponentAId");
    $hInformation{'ComponentBId'} = GetChildDataBySingleTagName($xmlNode, "ComponentBId");
    $hInformation{'Style'} = EmptyStringIfUndef(GetChildDataBySingleTagName($xmlNode, "Style"));
    $hInformation{'ConnectionTitle'} = EmptyStringIfUndef(GetChildDataBySingleTagName($xmlNode, "ConnectionTitle"));
    # TODO N Add the rest of the elements in the ComponentRelation structure.

    return(%hInformation);
} # end mdlGetComponentRelationInformation

# -----------------------------------------------------------------------
# Returns the name of the component for the given Component ID.
# @param nId: Id of the Component, to look-up.
# @return Name of the component, if found, otherwise undef.
#
# @see GetComponentInformation
# ---------------------
sub mdlGetComponentNameFromId {
	my $nId = shift || confess("!!! You must provide a Component ID as the first parameter");

	my $szReturnString;

    Tee("III                mdlGetComponentNameFromId($nId)\n");

	my %hInformation = GetComponentInformation($nId);


    $szReturnString = $hInformation{'Name'};

    return($szReturnString);
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
    my $szViewPacketTypeName = shift;
    my $szViewPacketStyleName = shift;
    my $szLevelLimit = shift || 1000;

    my $szModuleIncludeName;

    if ( ! IsNumber($nId) ) {
      confess("EEE nId was not a number: '$nId' for PacketType: $szViewPacketTypeName");
    }

    # TODO C Does the CnC need to be sorted on the Relation type? Does CnC have a relation type?.
    my %hComponentHashList = GenerateCncDiagramAndLegend($nId, $szViewPacketStyleName, $szLevelLimit);

    $hComponentHashList{'szViewPacketTypeName'} = $szViewPacketTypeName;
    $hComponentHashList{'szViewPacketStyleName'} = $szViewPacketStyleName;

    #print Dumper(%hComponentHashList);
    $hComponentHashList{'GraphVizRankSeperation'} = 6; # TODO C Make this programmable.
    my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/Graphviz_${szViewPacketTypeName}Diagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";

    my $szNameWithoutSuffix = "ComponentDiagramTree${szViewPacketTypeName}${szViewPacketStyleName}${nId}";
    open(GRAPHVIZ, ">${szNameWithoutSuffix}.gv") or die("!!! Unable to create $szViewPacketTypeName diagram file ${szNameWithoutSuffix}.gv: $!");
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

    open(LATEX, ">${szNameWithoutSuffix}.tex") or die("!!! Unable to create $szViewPacketTypeName diagram file: $!");
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

    # The f_nComponentRecursingLevelLeft will be decremented by GenerateCncDiagramAndLegendRecursively().
    $f_nComponentRecursingLevelLeft = $szLevelLimit;

    # Arbitrary number to make it 'unlimited'.

    Log("III GenerateCncDiagramAndLegend($nComponentId, $szRelationType)\n");


    GenerateCncDiagramAndLegendRecursively($nComponentId, $szRelationType,  $f_nComponentRecursingLevelLeft);
} # GenerateCncDiagramAndLegend

# -----------------------------------------------------------------
# ---------------
sub GenerateCncDiagramAndLegendRecursively {
    my $nComponentId = shift;
    my $szRelationType = shift;
    my $nComponentRecursingLevelLeft = shift;

#   $f_nComponentRecursingLevelLeft -= 1;
    Log("III    GenerateCncDiagramAndLegendRecursively($nComponentId, $szRelationType), ($nComponentRecursingLevelLeft)\n");

    my %hReturnComplexHashesList;
    $hReturnComplexHashesList{'arEntityList'} = ();
    $hReturnComplexHashesList{'arRelationList'} = ();

    if ( $nComponentRecursingLevelLeft  >= 0 ) {
        # TODO Extract info for the nComponentId here
        my %hComponentInformation = GetComponentInformation($nComponentId);
        $hComponentInformation{'Id'} = $nComponentId;
        push(@{$hReturnComplexHashesList{'arEntityList'}}, \%hComponentInformation);
        if ( $nComponentRecursingLevelLeft  > 0 ) {
            my @arSubComponentHashList = GetHashListRelatedComponent($nComponentId,  $szRelationType);
            foreach my $refhSumComponentStructure(@arSubComponentHashList) {
                # Create the relation here, how do I make it (uses) generically?
                my %hRelation;
                $hRelation{ 'EntityAId'} = $nComponentId;
                $hRelation{ 'EntityBId'} = $refhSumComponentStructure->{'RelatedId'};
                $hRelation{ 'RelationType'}    = $szRelationType;
                $hRelation{ 'Name'}    = ""; # TODO V make this dependent on the relation type, or maybe make it empty.

                $hRelation{'Direction'} = "";
                if ( defined($refhSumComponentStructure->{'PropertiesOfRelation'}) ) {
                  if ( $refhSumComponentStructure->{'PropertiesOfRelation'} eq "RO" ) {
                    $hRelation{'Direction'} = "in";
                  } elsif ( $refhSumComponentStructure->{'PropertiesOfRelation'} eq "WO" ) {
                    $hRelation{'Direction'} = "out";
                  } elsif ( $refhSumComponentStructure->{'PropertiesOfRelation'} eq "RW" ) {
                    $hRelation{'Direction'} = "both";
                  }
                } # endif defined.

                push(@{$hReturnComplexHashesList{'arRelationList'}}, \%hRelation);

                my %hTmpHash = GenerateCncDiagramAndLegendRecursively($refhSumComponentStructure->{'RelatedId'}, $szRelationType, $nComponentRecursingLevelLeft-1);
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
# returns a hash with two entries:
#  arEntityList: Array of %hComponentInformation
#  arRelationList: Array of
# ---------------
sub GenerateComponentDiagramAndLegendRecursively {
   my $nComponentId = shift || confess("!!! You must provide a component ID as the first parameter");
   my $szViewPacketTypeName = shift || confess("!!! You must provide a view packet type name as parm 2.");
   my $szViewPacketStyleName = shift || confess("!!! You must provide a view packet style name as parm 3.");
   my $nComponentRecursingLevelLeft = shift;

   # $f_nComponentRecursingLevelLeft -= 1; # TODO V Should this be re-activated?
   Log("III             GenerateComponentDiagramAndLegendRecursively($nComponentId, $szViewPacketTypeName, $szViewPacketStyleName), ($nComponentRecursingLevelLeft)\n");

   my %hReturnComplexHashesList;
   $hReturnComplexHashesList{'arEntityList'} = ();
   $hReturnComplexHashesList{'arRelationList'} = ();

  if ( $nComponentRecursingLevelLeft  >= 0 ) {
    # Extract info for the nComponentId here
    my %hComponentInformation = GetComponentInformation($nComponentId);
    push(@{$hReturnComplexHashesList{'arEntityList'}}, \%hComponentInformation);
    if ( $nComponentRecursingLevelLeft  > 0 ) {
      my @arSubComponentIdList = GetSubComponentList($nComponentId, $szViewPacketTypeName, $szViewPacketStyleName);
      #print "III             GenerateComponentDiagramAndLegendRecursively($nComponentId, $szStyleName), ($nComponentRecursingLevelLeft) after call to GetSubComponentList() ar sz: $#arSubComponentIdList\n";
      foreach my $nSubComponentId (@arSubComponentIdList) {
        # Create the relation here, how do I make it (uses) generically?
        my %hRelation;
        $hRelation{ 'EntityAId'} = $nComponentId;
        $hRelation{ 'EntityBId'} = $nSubComponentId;
        $hRelation{ 'RelationType'}    = $szViewPacketStyleName;

        # TODO V Support ConnectorId
        # This is should be replaced with a ConenctorTypeList that should be stored in a hash,
        #   Multiple ComponentRelations could use the same connector type, like TCP etc.
        $hRelation{ 'Name'}    = ""; # TODO V make this dependent on the relation type, or maybe make it empty.
        $hRelation{ 'Summary'}    = ""; # TODO V make this dependent on the relation type, or maybe make it empty.

        $hRelation{'Direction'} = "";
        push(@{$hReturnComplexHashesList{'arRelationList'}}, \%hRelation);

        my %hTmpHash = GenerateComponentDiagramAndLegendRecursively($nSubComponentId, $szViewPacketTypeName, $szViewPacketStyleName, $nComponentRecursingLevelLeft-1);
        push(@{$hReturnComplexHashesList{'arEntityList'}}, @{$hTmpHash{'arEntityList'}});
        if ( exists($hTmpHash{'arRelationList'})  && defined($hTmpHash{'arRelationList'}) ) {
          push(@{$hReturnComplexHashesList{'arRelationList'}}, @{$hTmpHash{'arRelationList'}});
        }
      } # end foreach.
      #print Dumper(@arSubComponentHashList);
      #print "DDD $#arSubComponentHashList\n";
    } # endif >0
  } # endif.
  #print "DDD done GenerateComponentDiagramAndLegendRecursively($#arSubComponentHashList)\n";
  return(%hReturnComplexHashesList);
} # end GenerateComponentDiagramAndLegendRecursively.


# -----------------------------------------------------------------
# ---------------
sub  GenerateComponentDiagramAndLegend {
    my $nComponentId = shift || confess("!!! You must provide a component ID as the first parameter");
    my $szViewPacketTypeName = shift || confess("!!! You must provide a view packet type name as parm 2.");
    my $szViewPacketStyleName = shift || confess("!!! You must provide a view packet style name as parm 3.");
    my $szLevelLimit = shift || 1000;

    # The f_nComponentRecursingLevelLeft will be decremented by GenerateComponentDiagramAndLegendRecursively().
    $f_nComponentRecursingLevelLeft = $szLevelLimit;

    # Arbitrary number to make it 'unlimited'.

    Log("III          GenerateComponentDiagramAndLegend($nComponentId, $szViewPacketTypeName, $szViewPacketStyleName, $szLevelLimit)\n");

    my %hComponentHashList = GenerateComponentDiagramAndLegendRecursively($nComponentId, $szViewPacketTypeName, $szViewPacketStyleName,  $f_nComponentRecursingLevelLeft);
    #Tee("DDD          GenerateComponentDiagramAndLegend($nComponentId, $szViewPacketTypeName, $szViewPacketStyleName) after GenerateComponentDiagramAndLegendRecursively() call.\n");

    return(%hComponentHashList);
} # GenerateComponentDiagramAndLegend



# -----------------------------------------------------------------------
# ---------------------
sub GenerateModuleDecompositionViewPacket {
 my $nId = shift;

  $f_nViewPacketNumber = 0;
  my $szContent = GenerateViewPacketRecusrsively($nId);

  Log("------------------------------------------\n");

  my $szFileName = "moduleDecompositionViewPackets";
  open(FINISHED_DOC, ">${szFileName}.tex") or die("!!! Unable to open '${szFileName}.tex' for write: $!");
  print FINISHED_DOC $szContent;
  close(FINISHED_DOC);

}




# -----------------------------------------------------------------------
# TODO V possibly add 'AndLegend' to the end of the function name.
# Gets the Diagram tree for the given component
#  nId: The Component name.
#  szTypeName: e.g. Module
#  szStyleName:  E.g. Decomposition.
#  szLevelLimit:
# Returns: szNameWithoutSuffix
# ---------------------
sub GenerateComponentDiagramTree {
    my $nId = shift;
    my $szViewPacketTypeName = shift; # This is just here to optionally make it easier to merge with other view types later.
    my $szViewPacketStyleName = shift || confess("!!! You must specify a style name.");
    my $szLevelLimit = shift || 1000;

    my $szModuleIncludeName;
    Log("III       GenerateComponentDiagramTree($nId, $szViewPacketTypeName, $szViewPacketStyleName, $szLevelLimit)\n");

    # TODO C how can this be fixed to szDecompositionStyleName
    my %hComponentHashList = GenerateComponentDiagramAndLegend($nId, $szViewPacketTypeName, $szViewPacketStyleName);

    $hComponentHashList{'szViewPacketTypeName'} = $szViewPacketTypeName;
    $hComponentHashList{'szViewPacketStyleName'} = $szViewPacketStyleName;

    #print Dumper(\%hComponentHashList);
    $hComponentHashList{'GraphVizRankSeperation'} = 6; # TODO C Make this programmable.
    #print "DDD using template: ${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl\n";
    my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";

    # TODO C Can we ever come accross needing two diagrams of the same component within the same type and style?
    my $szNameWithoutSuffix = "ComponentDiagramTree${szViewPacketTypeName}${szViewPacketStyleName}${nId}";
    open(GRAPHVIZ, ">${szNameWithoutSuffix}.gv") or die("!!! Unable to create module diagram file ${szNameWithoutSuffix}.gv: $!");
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

    open(LATEX, ">${szNameWithoutSuffix}.tex") or die("!!! Unable to create module diagram file: $!");
    print LATEX "$szResult\n";
    close(LATEX);

    # update Makefile
    # Target: Dependency  \n Command
    UpdateSubMakefile("${szNameWithoutSuffix}.ps", "${szNameWithoutSuffix}.gv", "\tdot -Tps -o \$\@ \$^");
    UpdateSubMakefile("${szNameWithoutSuffix}.png", "${szNameWithoutSuffix}.gv", "\tdot -Tpng -o \$\@ \$^");

    return($szNameWithoutSuffix);
} # end GenerateComponentDiagramTree



# -----------------------------------------------------------------------
# TODO C what is the difference between this and the ...Tree function above?
# TODO C Put this as call in the Viewpacket gen thing.
# TODO C somehow coordinate the filename, possibly by returning it from here.
# ---------------------
sub GeneratemoduleComponentDiagram {
    my $nId = shift;

confess("XXX Who uses this function?");

    my %hComponentHashList = GenerateModuleComponentDiagramAndLegend($nId, "$f_szDecompositionStyleName", 1);
#     print Dumper(%hComponentHashList);
    $hComponentHashList{'GraphVizRankSeperation'} = 6; # TODO C Make this programmable.
    Log("DDD using template: ${f_szAbsolutePathToCurrentScript}/Graphviz_componentDiagram.tmpl\n");
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
  Tee(Dumper(\%hProjectExecutiveCharter));
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

  #Tee("DDD GetSipoc(${nId})\n");

  my $xmlElement = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "Sipoc", "Id", $nId);
  if ( defined($xmlElement) ) {
    $hTemporary{"Headline"} = $xmlElement->getAttribute("Headline");
    push(@{$hTemporary{"Supplier"}}, GetDataArrayByTagName($xmlElement, "Supplier") );
    push(@{$hTemporary{"Input"}}, GetDataArrayByTagName($xmlElement, "Input") );
Tee("XXX TODO C Implement GetArrayOfContentForEachElementByTagSortedByAttribute() in the Xml.pm\n");
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

# -----------------------------------------------------------------
# ---------------
sub GenerateViewPacketByTypeAndStyle {
  my $xmlNode               = shift;
  my $refhSwArcHashList     = shift;
  my $szViewPacketTypeName  = shift;
  my $szViewPacketStyleName = shift;

  # TODO N Verify that the refhSwArcHashList is actually a hash reference.

  confess("EEE You must provide an XML node.") unless(defined($xmlNode));
  confess("EEE You must provide a packet Type and Style.") unless( defined($szViewPacketTypeName) && defined($szViewPacketStyleName) );


  my @arViewPacketFileList;
  my @arViewPacketList = GetDataArrayByTagName($xmlNode, "${szViewPacketTypeName}${szViewPacketStyleName}ViewPacketId");
  Tee("III GenerateViewPacketByTypeAndStyle(xmlNode, refhSwArcHashList, $szViewPacketTypeName, $szViewPacketStyleName) arViewPacketList=$#arViewPacketList\n");
  # TODO V Sort the arViewPacketList by the 'Order' attribute.
  foreach my $nViewPacketId (@arViewPacketList) {
    #Tee("DDD nViewPacketId=$nViewPacketId\n");
    if ( IsNumber($nViewPacketId) ) {
      $nViewPacketId += 0;
    } else {
      $nViewPacketId = 0;
    }
    if ( $nViewPacketId > 0 ) {
        my $szText = GenerateViewPacketRecusrsively($nViewPacketId);
        my $szViewPacketFileName = "Viewpacket${szViewPacketTypeName}${szViewPacketStyleName}${nViewPacketId}";
        #Tee("DDD Writing nViewPacketId: $nViewPacketId information to: $szViewPacketFileName\n");
        push(@arViewPacketFileList, $szViewPacketFileName);
        open(LATEX, ">${szViewPacketFileName}.tex") || die("EEE unable to open file for write '${szViewPacketFileName}.tex': $!");
        print LATEX $szText;
        close(LATEX);
    }
  }

  $refhSwArcHashList->{"ar${szViewPacketTypeName}${szViewPacketStyleName}ViewPackets"} = \@arViewPacketFileList;

#  return(@arViewPacketFileList);
} # end GenerateViewPacketByTypeAndStyle


# -----------------------------------------------------------------
# ---------------
sub IncludeContextModelIfExists {
  my $xmlNode = shift;
  my $refhHash = shift;

  confess("EEE Missing parameters") unless (defined($refhHash));

  my $nContextModelId = GetChildDataBySingleTagName($xmlNode, "ContextModelId");

  if ( IsNumber($nContextModelId) ) {
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
# @see GenerateSoftwareArchitectureDocumentation
# @see GenerateViewPacketRecusrsively
# @see GenerateUmlDiagram
# ---------------
sub IncludeDiagramIfExists {
  my $xmlNode = shift;
  my $refhHash = shift;

  confess("EEE Missing parameters") unless (defined($refhHash));

  my $nDiagramId = GetChildDataBySingleTagName($xmlNode, "DiagramId");

  if ( IsNumber($nDiagramId) ) {
    $nDiagramId += 0;
  }  else {
    $nDiagramId = 0;
  }
  if ( $nDiagramId > 0 ) {
    my $szFileNameWithSuffix =  GenerateUmlDiagram($nDiagramId);
    my($szFileName, $directories, $szSuffix) = fileparse($szFileNameWithSuffix, qr/\.[^.]*/);
    # TODO C How do I find out, which section to put this diagram in?
    # TODO C I think this has to be made into an array that is pushed to, since there can be multiple diagrams.
    $refhHash->{'DiagramFileName'} = $szFileName;
  } else {
    $refhHash->{'DiagramFileName'} = $f_szNotPresent;
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
  IncludeDiagramIfExists($xmlNode, \%hProject);


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
  # Note: Remember to add the chapter to the LaTeX_SwArdDoc.tmpl, when you add it here.
  # Modules
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szModuleTypeName, $f_szDecompositionStyleName);
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szModuleTypeName, $f_szUsesStyleName);
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szModuleTypeName, $f_szGeneralizationStyleName);

  # CnC: Component and Connectors
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szComponentAndConnectorTypeName, $f_szCommunicatingProcessesStyleName);
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szComponentAndConnectorTypeName, $f_szSharedDataStyleName);

  # Allocation
  GenerateViewPacketByTypeAndStyle($xmlNode, \%hSwArcHashList, $f_szAllocationTypeName, $f_szImplementationStyleName);


  Tee(Dumper(\%hSwArcHashList));

  Tee("III Populating   LaTeX_SwArcDoc.tmpl\n");
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_SwArcDoc.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";
   my $szResult = $template->fill_in(HASH => \%hSwArcHashList);

  #Tee("DDD Writing content to sad.tex.\n");
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

  if ( ! IsNumber($nViewPacketId) ) {
    confess("EEE nViewPacketId was not a number: '$nViewPacketId'");
  }

  $f_nViewPacketNumber += 1;
  my %hModuleInformation;

  #
  $hModuleInformation{ReferenceLabel} = "labelViewPacket$nViewPacketId";

  # Get the XML node for the view packet with Id: $nViewPacketId
  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $nViewPacketId);

  $hModuleInformation{PacketTitle} = $xmlNode->getAttribute("Title");
  #print "DDD $hModuleInformation{PacketTitle}\n";
  my $szViewPacketTypeName = GetChildDataBySingleTagName($xmlNode, "ViewPacketType");
  $hModuleInformation{ViewPacketType} = $szViewPacketTypeName;
  my $szViewPacketStyleName = GetChildDataBySingleTagName($xmlNode, "ViewPacketStyle");
  $hModuleInformation{ViewPacketStyle} = $szViewPacketStyleName;
  $hModuleInformation{Introduction} = GetChildDataBySingleTagName($xmlNode, "Introduction");
  my $nViewPackeSpecefiedComponentRecurseLevel = GetChildDataBySingleTagName($xmlNode, "ComponentRecurseLevel");

  if ( IsNumber($nViewPackeSpecefiedComponentRecurseLevel) ) {
    $szComponentLevelLimit = $nViewPackeSpecefiedComponentRecurseLevel;
  }


  Tee("III    GenerateViewPacketRecursively(${nViewPacketId}, ${szViewPacketTypeName} - $szViewPacketStyleName) (f_nViewPacketNumber=$f_nViewPacketNumber) Current recurselevel: $szComponentLevelLimit\n");

  $hModuleInformation{PacketNumber} = $f_nViewPacketNumber;

  my $nPrimaryPresentationComponentId =   GetChildDataBySingleTagName($xmlNode, "ComponentId");
  if ( IsNumber($nPrimaryPresentationComponentId) ) {
    # TODO V the 'module' part of the name should be derived from what view type and style is being generated.
    if ( ( $szViewPacketTypeName eq $f_szModuleTypeName ) || ( $szViewPacketTypeName eq $f_szAllocationTypeName ) ) {
      $hModuleInformation{'PrimaryPresentationNameFile'}= GenerateComponentDiagramTree($nPrimaryPresentationComponentId, $szViewPacketTypeName, $szViewPacketStyleName, $szComponentLevelLimit);
    } elsif ( $szViewPacketTypeName eq $f_szComponentAndConnectorTypeName ) {
      $hModuleInformation{'PrimaryPresentationNameFile'}= GenerateCncDiagramTree($nPrimaryPresentationComponentId, $szViewPacketTypeName, $szViewPacketStyleName, $szComponentLevelLimit);
    } else {
      confess("EEE View packet type($szViewPacketTypeName) not yet supported. (nViewPacketId=${nViewPacketId}) please implement: $szViewPacketTypeName");
    }
    # TODO C write gen command to the subcomponnent make file.
  } else {
    $hModuleInformation{'PrimaryPresentationNameFile'} = undef;
    confess("EEE No 'ComponentId' found in 'ViewPacket'  Id = $nViewPacketId");
  }

  IncludeContextModelIfExists($xmlNode, \%hModuleInformation);
  IncludeDiagramIfExists($xmlNode, \%hModuleInformation);

  # Get list of all 'ModuleRelations'
  # Filter on RelationType=$f_szDecompositionStyleName   and  ModuleBId=$nTopPacketId
  # TODO C The '$f_szDecompositionStyleName' is dependent on the szViewType
  my @arSubComponentIdList = GetSubComponentList($nPrimaryPresentationComponentId, $szViewPacketTypeName, $szViewPacketStyleName);

  my @arSubComponentHashList;
  foreach my $nComponentId (@arSubComponentIdList) {
      my %hComponentInformation = GetComponentInformation($nComponentId);
      push(@arSubComponentHashList, \%hComponentInformation);
  }
  $hModuleInformation{ComponentList} = \@arSubComponentHashList;
  # print Dumper(@arSubComponentHashList);
  # $hModuleInformation{TopModule} = \%{GetComponentInformation($nTopModuleId)};
  my %hTmp = GetComponentInformation($nPrimaryPresentationComponentId);
  # print Dumper(\%hTmp);
  # TODO V Do I still use this information?
  $hModuleInformation{TopModule} = \%hTmp;
  # print Dumper($hModuleInformation{TopModule});

  my @arParentViewPacketList = GetViewPacketLabelAndTitleHashList($nViewPacketId, "Parent" );
  $hModuleInformation{ParentViewPacketHashList} = \@arParentViewPacketList;
  my @arChildrenViewPacketList = GetViewPacketLabelAndTitleHashList($nViewPacketId, "Child" );
  $hModuleInformation{ChildViewPacketList} = \@arChildrenViewPacketList;
  my @arSiblingViewPacketList = GetViewPacketLabelAndTitleHashList($nViewPacketId, "Sibling" );
  $hModuleInformation{SiblingViewPacketList} = \@arSiblingViewPacketList;




  #my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_${szViewPacketType}${szViewPacketStyle}ViewPacket.tmpl")
  # Attempting to have one template for all view packet types.
  my $template = Text::Template->new(TYPE => 'FILE', SOURCE => "${f_szAbsolutePathToCurrentScript}/LaTeX_ViewPacket.tmpl")
            or die "Couldn't construct template: $Text::Template::ERROR";

  my $szResult = $template->fill_in(HASH => \%hModuleInformation);
  if ( ! defined($szResult) ) {
    Tee(Dumper(\%hModuleInformation));
  }

  # TODO V The $? is worthless at this point
  confess("EEE Template fill failed for LaTeX_ViewPacket.tmpl (template op, rc: $?)\n    Template error: '$Text::Template::ERROR')") unless(defined($szResult));

  my @arIdList =  GetDataArrayByTagName($xmlNode, "ChildViewPacketId");

  foreach my $nChildPacketViewId (@arIdList) {
      #Tee("DDD nChildPacketViewId = $nChildPacketViewId\n");
      #print Dumper(\@arIdList);
      if (  $nChildPacketViewId  =~ /\d+/ ) {
         $nChildPacketViewId += 0;
        my $nComponentLevelsToShowInPrimaryPresentation = 1;
        #Tee("DDD    nViewPacketId=${nViewPacketId} now calling: GenerateViewPacketRecusrsively($nChildPacketViewId, \"DUMMY\", $nComponentLevelsToShowInPrimaryPresentation)\n");
        my $szChildText = GenerateViewPacketRecusrsively($nChildPacketViewId, "DUMMY", $nComponentLevelsToShowInPrimaryPresentation);
        confess("EEE No text generated by: GenerateViewPacketRecusrsively($nChildPacketViewId, DUMMY, $nComponentLevelsToShowInPrimaryPresentation)") unless(defined($szChildText));
        $szResult .= $szChildText;
      } # end if.
  } # end foreach.


  return($szResult);
} # end GenerateViewPacketRecusrsively.


# -----------------------------------------------------------------
#  If the szInputString is undefined, then return an empty string.
#   otherwise return the szInputString.
# ---------------
sub EmptyStringIfUndef {
  my $szInputString = shift;

  my $szAnswer = "";

  if (defined($szInputString)) {
    $szAnswer = $szInputString;
  }

  return($szAnswer);
} # end EmptyStringIfUndef.

# -----------------------------------------------------------------
# ---------------
sub GetComponentInformation {
    my $nComponentId = shift;

    my %hComponentInformation;

    my $szTagNameToFind = "Component";

    Log("III                GetComponentInformation($nComponentId)\n");


    my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, $szTagNameToFind,  "Id", "$nComponentId");
    confess("EEE Unable to get node for Tag: '$szTagNameToFind' and Id = $nComponentId") unless(defined($xmlNode));

    $hComponentInformation{'Id'} = $nComponentId;
    $hComponentInformation{'Name'} = $xmlNode->getAttribute("Name");
    $hComponentInformation{'Summary'} = GetChildDataBySingleTagName($xmlNode, "Summary");
    $hComponentInformation{'Responsibilities'} = GetChildDataBySingleTagName($xmlNode, "Responsibilities");
    $hComponentInformation{'BehaviorDescription'} = GetChildDataBySingleTagName($xmlNode, "BehaviorDescription");
    $hComponentInformation{'Interfaces'} = GetChildDataBySingleTagName($xmlNode, "Interfaces");
    $hComponentInformation{'SourceRepository'} = GetChildDataBySingleTagName($xmlNode, "SourceRepository");
    $hComponentInformation{'ArtifactRepository'} = GetChildDataBySingleTagName($xmlNode, "ArtifactRepository");
    $hComponentInformation{'BuildCycle'} = GetChildDataBySingleTagName($xmlNode, "BuildCycle");
    $hComponentInformation{'Contact'} = EmptyStringIfUndef(GetChildDataBySingleTagName($xmlNode, "Contact"));
    $hComponentInformation{'License'} = EmptyStringIfUndef(GetChildDataBySingleTagName($xmlNode, "License"));
#    $hComponentInformation{} = GetChildDataBySingleTagName($xmlNode, "");

#    print Dumper(\%hComponentInformation);

    return(%hComponentInformation);
} #end GetComponentInformation


# -----------------------------------------------------------------
# ---------------
sub GetViewPacketInformation {
    my $nViewPacketId = shift || confess("!!! You must provide a view packet ID as the first parameter");

    my %hViewPacketInformation;

    my $szTagNameToFind = "ViewPacket";

    Tee("III                GetViewPacketInformation($nViewPacketId)\n");


    my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, $szTagNameToFind,  "Id", "$nViewPacketId");
    confess("EEE Unable to get node for Tag: '$szTagNameToFind' and Id = $nViewPacketId") unless(defined($xmlNode));

    $hViewPacketInformation{'Id'} = $nViewPacketId;
    $hViewPacketInformation{'Title'} = $xmlNode->getAttribute("Title");
    $hViewPacketInformation{'Introduciton'} = GetChildDataBySingleTagName($xmlNode, "Introduction");
    $hViewPacketInformation{'ComponentId'} = GetChildDataBySingleTagName($xmlNode, "ComponentId");
    $hViewPacketInformation{'ViewPacketType'} = GetChildDataBySingleTagName($xmlNode, "ViewPacketType");
    $hViewPacketInformation{'ViewPacketStyle'} = GetChildDataBySingleTagName($xmlNode, "ViewPacketStyle");
#    $hViewPacketInformation{''} = GetChildDataBySingleTagName($xmlNode, "");

#    print Dumper(\%hViewPacketInformation);

    return(%hViewPacketInformation);
} #end GetViewPacketInformation


# -----------------------------------------------------------------
# ---------------
sub GetHashListRelatedComponent {
    my $nCurrentComponentId = shift;
    my $szRequiredStyleType = shift;

    if ( ! IsNumber($nCurrentComponentId) ) {
      confess("EEE nTopPacketId was not a number: '$nCurrentComponentId' for TopPacketId: $nCurrentComponentId RelationType: $szRequiredStyleType");
    }

    #Tee("DDD                GetComponentRelationsList($nCurrentComponentId, $szRequiredStyleType)\n");

    my @arResponse;

    my $szRelationTagName = "ComponentRelation";

    my @arNodeList = GetNodeArrayByTagName($f_xmlRoot, $szRelationTagName);
    #Tee("DDD                   #arNodeList=$#arNodeList\n");

    foreach my $xmlNode (@arNodeList) {
      my $szEntityAId = GetChildDataBySingleTagName($xmlNode, "ComponentAId");
      my $szEntityBId = GetChildDataBySingleTagName($xmlNode, "ComponentBId");
      my $szStyleName = GetChildDataBySingleTagName($xmlNode, "Style");
      #Tee("DDD                   szEntityAId=$szEntityAId szEntityBId=$szEntityBId szStyleName=$szStyleName\n");
      if ( IsNumber($szEntityAId) ) {
        if ( ($szEntityAId == $nCurrentComponentId) && ( $szStyleName eq $szRequiredStyleType) ){
          my $szPropertiesOfRelation = GetChildDataBySingleTagName($xmlNode, "PropertiesOfRelation");
          my %hRelation = ( 'RelatedId' => $szEntityBId,
                            'PropertiesOfRelation' => $szPropertiesOfRelation
                          );
          push(@arResponse, \%hRelation);
        } # endif
      } else {
        my $nModuleRelationsId = $xmlNode->getAttribute("Id");
        warn("WWW ComponentAId was not a number: '$szEntityAId' for nCurrentComponentId=$nCurrentComponentId at <$szRelationTagName Id=\"$nModuleRelationsId\"> szRelationType: $szRequiredStyleType");
      }
    } # end foreach

    return(@arResponse);
} # end GetComponentInformation



# -----------------------------------------------------------------
# ---------------
sub GetComponentRelationsList {
    my $nCurrentComponentId = shift;
    my $szRequiredStyleType = shift;

    if ( ! IsNumber($nCurrentComponentId) ) {
      confess("EEE nTopPacketId was not a number: '$nCurrentComponentId' for TopPacketId: $nCurrentComponentId RelationType: $szRequiredStyleType");
    }

    #Tee("DDD                GetComponentRelationsList($nCurrentComponentId, $szRequiredStyleType)\n");

    my @arResponse;

    my $szRelationTagName = "ComponentRelation";

    my @arNodeList = GetNodeArrayByTagName($f_xmlRoot, $szRelationTagName);
    #Tee("DDD                   #arNodeList=$#arNodeList\n");

    foreach my $xmlNode (@arNodeList) {
      my $szEntityAId = GetChildDataBySingleTagName($xmlNode, "ComponentAId");
      my $szEntityBId = GetChildDataBySingleTagName($xmlNode, "ComponentBId");
      my $szStyleName = GetChildDataBySingleTagName($xmlNode, "Style");
      #Tee("DDD                   szEntityAId=$szEntityAId szEntityBId=$szEntityBId szStyleName=$szStyleName\n");
      if ( IsNumber($szEntityAId) ) {
        if ( ($szEntityAId == $nCurrentComponentId) && ( $szStyleName eq $szRequiredStyleType) ){
          push(@arResponse, $szEntityBId);
        } # endif
      } else {
        my $nModuleRelationsId = $xmlNode->getAttribute("Id");
        warn("WWW ComponentAId was not a number: '$szEntityAId' for nCurrentComponentId=$nCurrentComponentId at <$szRelationTagName Id=\"$nModuleRelationsId\"> szRelationType: $szRequiredStyleType");
      }
    } # end foreach

    return(@arResponse);
} # end GetComponentRelationsList

# -----------------------------------------------------------------
# ---------------
sub GetSubComponentList {
    my $nComponentId = shift || confess("!!! You must provide a ComponentId");
    my $szViewPacketTypeName = shift || confess("!!! You must provide Type a name");
    my $szViewPacketStyleName = shift || confess("!!! You must provide Style a name");

    my @arResponse;

#    if( $szViewPacketTypeName eq $f_szModuleTypeName ) {
#      @arResponse = GetModuleSubComponentList($nComponentId, $szViewPacketStyleName);
    if ( ( $szViewPacketTypeName eq $f_szComponentAndConnectorTypeName )
            || ( $szViewPacketTypeName eq $f_szModuleTypeName )
            || ( $szViewPacketTypeName eq $f_szSubTypeName )
            ) {
       @arResponse = GetComponentRelationsList($nComponentId, $szViewPacketStyleName);
    } elsif( $szViewPacketTypeName eq $f_szAllocationTypeName ) {
      # TODO C How should allocation relations look? ( Or should the Implement relation specify which substruture and depth to use?
      #    So it could also be a combination of e.g. Module-Decomposition and Module-Uses.
      #  Is there are difference between the allocation styles?
      #  Should it take the full recursive level set or just one level down?
      #  Is it Only Module-Uses that is relevant, or would Module-Decomposition also make sense? I think only Module-Uses
      #  Is Module-Layered also relevant?
      if ( $szViewPacketStyleName eq $f_szImplementationStyleName ) {
        @arResponse = GetModuleSubComponentList($nComponentId, $f_szUsesStyleName);
      } else {
        confess("Please implement the Get${szViewPacketTypeName}SubComponentList() to support: szViewPacketTypeName: $szViewPacketTypeName and szViewPacketStyleName: $szViewPacketStyleName");
      }
    } else {
      confess("Please implement the Get${szViewPacketTypeName}SubComponentList() to support: szViewPacketTypeName: $szViewPacketTypeName and szViewPacketStyleName: $szViewPacketStyleName");
    }
    return(@arResponse);
} # end GetSubComponentList().


# -----------------------------------------------------------------
# Provide a list of Component IDs the the nCurrentComponentId relates to with the given relation: szRequiredRelationType
#  This is not recursive, it simply returns the next layer.
# ---------------
sub GetModuleSubComponentList {
    my $nCurrentComponentId = shift;
    my $szRequiredRelationType = shift;

    #my $szCallTrace = shortmess("DDD call trace");
    #print "III                GetModuleSubComponentList($nTopPacketId, $szRequiredRelationType) szCallTrace: $szCallTrace\n";
    #Tee("III                GetModuleSubComponentList($nCurrentComponentId, $szRequiredRelationType)\n");

    my @arResponse;

    if ( IsNumber($nCurrentComponentId) ) {
      my @arNodeList = GetNodeArrayByTagName($f_xmlRoot, "ModuleRelations");

      foreach my $xmlNode (@arNodeList) {
        my $szModuleAId =  GetChildDataBySingleTagName($xmlNode, "ModuleAId");
        my $szModuleBId = GetChildDataBySingleTagName($xmlNode, "ModuleBId");
        my $szCurrentRelationType = GetChildDataBySingleTagName($xmlNode, "RelationType");

        # If the szModuleAId is not a number, then it is probably an empty xml structure.
        if ( IsNumber($szModuleAId) ) {
          if ( IsNumber($szModuleBId) ) {
            #
            if ( ($szModuleAId == $nCurrentComponentId) && ( $szRequiredRelationType eq $szCurrentRelationType) ){
              #Tee("DDD                ($szModuleAId == $nCurrentComponentId)  ( $szRequiredRelationType eq $szCurrentRelationType) => szModuleBId=$szModuleBId\n");
              push(@arResponse, $szModuleBId);
            } # endif
          } else {
	    my $nModuleRelationsId = $xmlNode->getAttribute("Id");
            warn("WWW szModuleBId was not a number: '$szModuleBId' for nTopPacketId=$nCurrentComponentId, ModuleRations Id: $nModuleRelationsId, szRequiredRelationType=$szRequiredRelationType");
          }
        } # endif AId
      } # end foreach
    } else {
	confess("WWW nCurrentComponentId was not a number: '$nCurrentComponentId'");
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

  Tee("III       GetViewPacketLabelAndTitleHashList($nViewPacketId, $szViewRelation)\n");
  my @arLabelReferenceList;

  # Get the node for the viewpacket.
  my $xmlNode = GetSingleChildNodeByTagAndAttribute($f_xmlRoot, "ViewPacket",  "Id", $nViewPacketId);

  my @arIdList =  GetDataArrayByTagName($xmlNode, "${szViewRelation}ViewPacketId");

  if ( $szViewRelation eq "Sibling" ) {
      my @arInherentSiblings = GetInherentSiblingList($nViewPacketId);
      push(@arIdList, @arInherentSiblings);
      Tee(Dumper(@arIdList)) unless($#arIdList == -1);
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
  if ( ( defined($szPossibleNumber)) && (  $szPossibleNumber  =~ /^\d+$/ ) ) {
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
  Tee(Dumper(\%hProject));

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
Tee("DDD Begin\n");

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
  GenerateComponentDiagramTree($nSectionId);
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


Tee("DDD End - last line\n");

close(LOG);
