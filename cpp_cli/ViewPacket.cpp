/*
 * ViewPacket.cpp
 *
 *  Created on: Sep 16, 2017
 *      Author: cadm
 */

#include "ViewPacket.h"
#include <assert.h>

#include "ComponentRelationLists.h"

ViewPacket::ViewPacket(DatabaseInterface* pDatabaseInterface, std::string sViewPacketId) {
	assert(pDatabaseInterface != NULL);
	assert(sViewPacketId.compare("") != 0);

	m_pDatabaseInterface = pDatabaseInterface;
	m_sViewPacketIndex = sViewPacketId;


}

ViewPacket::~ViewPacket() {
	// TODO Auto-generated destructor stub
}

std::string ViewPacket::GetTitle() {
	std::string sTitle = "";
	return(sTitle);
}

int ViewPacket::SaveSection(std::ofstream *pFile) {
	int nStatus = 0;

	std::string sTitle = m_pDatabaseInterface->GetViewPacketElementText(m_sViewPacketIndex, "Title");
	std::string sViewPacketType = m_pDatabaseInterface->GetViewPacketElementText(m_sViewPacketIndex, "ViewPacketType");
	std::string sViewPacketStyle = m_pDatabaseInterface->GetViewPacketElementText(m_sViewPacketIndex, "ViewPacketStyle");
	std::string sReferenceLabel = "labelViewPacket" + m_sViewPacketIndex;

	std::string sIntroduction = m_pDatabaseInterface->GetViewPacketElementText(m_sViewPacketIndex, "Introduction");
	std::string sComponentId = m_pDatabaseInterface->GetViewPacketElementText(m_sViewPacketIndex, "ComponentId");
	std::string sComponentRecurseLevel = m_pDatabaseInterface->GetViewPacketElementText(m_sViewPacketIndex, "ComponentRecurseLevel");

	int nComponentRecurseLevel = std::stoi(sComponentRecurseLevel);

	ComponentRelationLists *pComponentTree = m_pDatabaseInterface->GetComponentTree(sComponentId, nComponentRecurseLevel);

	// TODO C call GenerateComponentDiagramTree / GenerateCncDiagramTree
	//   Calls: GenerateComponentDiagramAndLegend
	//    Calls: GenerateComponentDiagramAndLegendRecursively
	//      Get sub components for this style: GetSubComponentList($nComponentId, $szViewPacketTypeName, $szViewPacketStyleName);

	/*
	 * Graphviz:
	 *   List of enities (components)
	 *   List of relations and the direction
	 *     refhRelation->{Direction} : PropertiesOfRelation
	 */


	*pFile << "\\section{"<< sViewPacketType <<" " << sViewPacketStyle << " view packet " << m_sViewPacketIndex << ": " << sTitle << "}" << std::endl;
	*pFile << "\\label{" << sReferenceLabel << "}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << sIntroduction << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection*{Primary Presentation}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "% TODO V Include a picture, see AppendixA, p29" << std::endl;
	*pFile << "\\input{{$PrimaryPresentationNameFile}}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item {$TopModule{'Name'}}" << std::endl;
	*pFile << "  \\begin{itemize}" << std::endl;
	*pFile << "{ if ( $#ComponentList == -1 ) {" << std::endl;
	*pFile << "  $OUT .= \"\\item NONE.\\n\";" << std::endl;
	*pFile << "  } else {" << std::endl;
	*pFile << "   foreach my $hComponent (@ComponentList) {" << std::endl;
	*pFile << "     $OUT .= \"\\item $hComponent->{Name}\\n\";" << std::endl;
	*pFile << "    }" << std::endl;
	*pFile << "  }" << std::endl;
	*pFile << "}" << std::endl;
	*pFile << " \\end{itemize}" << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection*{Context diagram}" << std::endl;
	*pFile << "\\input{{$ContextModelFileName}}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection*{Element Catalog}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsubsection*{Elements and their Properties}" << std::endl;
	*pFile << "Properties of the {$TopModule{'Name'}} sub modules are:" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item Name, given in the following table." << std::endl;
	*pFile << "\\item Responsibility, given in the following table." << std::endl;
	*pFile << "\\item Visibility: TODO C calculate this." << std::endl;
	*pFile << "\\item Implementation information: TODO C calculate this. " << std::endl;
	*pFile << "\\end{itemize}" << std::endl;

	*pFile << "" << std::endl;
	*pFile << "% TODO C provide a table here or a description list, I think I like the later better." << std::endl;
	*pFile << "{foreach my $hComponent (@ComponentList) {" << std::endl;
	*pFile << " $OUT .= \"\\subparagraph{$hComponent->{Name}} $hComponent->{Summary}.\n$hComponent->{Responsibilities}\\n\";" << std::endl;
	*pFile << "  }" << std::endl;
	*pFile << "}" << std::endl;

	*pFile << "" << std::endl;
	*pFile << "\\subsubsection*{Relations and their properties}" << std::endl;
	*pFile << "% TODO V Make the relation, a variable, so that it can be changed or perhaps make this whole subsection a var." << std::endl;
	*pFile << "The relation type in view is {\em is-part-of}. There are no exceptions or additions to the relations shown in the primary presentation." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsubsection*{Element interfaces}" << std::endl;
	*pFile << "Element interfaces for segments are given in subsecquent decompositions." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsubsection*{Element Behavior}" << std::endl;
	*pFile << "TODO V Some elements have complex interactions with their environment. For purposes of understanding or analysis it is oftern incumbent on the architect to specify element behavior." << std::endl;
	*pFile << "" << std::endl;

	*pFile << "\\subsection*{Variability Guide}" << std::endl;
	*pFile << "TODO V put something here, first figure out what it is that should go here." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection*{Architecture background}" << std::endl;
	*pFile << "\\subsubsection{Design rationale}" << std::endl;
	*pFile << "\\subsubsection{Results of analysis}" << std::endl;
	*pFile << "TODO V is this like pugh matrixes etc?" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsubsection*{Assumptions}" << std::endl;
	*pFile << "TODO V define (What are assumptions?)" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection*{Other Information}" << std::endl;
	*pFile << "\\subsection*{Related view packets}" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "{   if ( $#ParentViewPacketHashList == -1 ) {" << std::endl;
	*pFile << "    $OUT .= \"\\item Parent: None\\n\";" << std::endl;
	*pFile << "  } else {" << std::endl;
	*pFile << "   my $szReference = $ParentViewPacketHashList[0];" << std::endl;
	*pFile << "    $OUT .= \"\\item Parent: $szReference->{Title} (Section \\ref{$szReference->{Label}}, page \\pageref{$szReference->{Label}})\\n\";" << std::endl;
	*pFile << "  }" << std::endl;
	*pFile << "}{   if ( $#ChildViewPacketList == -1 ) {" << std::endl;
	*pFile << "    $OUT .= \"\\item Children: None\\n\";" << std::endl;
	*pFile << " } else {" << std::endl;
	*pFile << "    $OUT .= \"\\item Children:\\n\";" << std::endl;
	*pFile << "    $OUT .= \"\\begin{itemize}\\n\";" << std::endl;
	*pFile << "    foreach my $szReference (@ChildViewPacketList) {" << std::endl;
	*pFile << "      $OUT .= \"\\item $szReference->{Title} (Section \\ref{$szReference->{Label}}, page \\pageref{$szReference->{Label}})\\n\";" << std::endl;
	*pFile << "    }" << std::endl;
	*pFile << "     $OUT .= \"\\end{itemize}\\n\";" << std::endl;
	*pFile << "  }" << std::endl;
	*pFile << "}{   if ( $#SiblingViewPacketList == -1 ) {" << std::endl;
	*pFile << "   $OUT .= \"\\item Siblings: None\\n\";" << std::endl;
	*pFile << "  } else {" << std::endl;
	*pFile << "   $OUT .= \"\\item Siblings:\\n\";" << std::endl;
	*pFile << "    $OUT .= \"\\begin{itemize}\\n\";" << std::endl;
	*pFile << "   foreach my $szReference (@SiblingViewPacketList) {" << std::endl;
	*pFile << "     $OUT .= \"\\item $szReference->{Title} (Section \\ref{$szReference->{Label}}, page \\pageref{$szReference->{Label}})\\n\";" << std::endl;
	*pFile << "  }" << std::endl;
	*pFile << "   $OUT .= \"\\end{itemize}\\n\";" << std::endl;
	*pFile << " }" << std::endl;
	*pFile << "}\end{itemize}" << std::endl;

	return(nStatus);
}
