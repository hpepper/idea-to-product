/*
 * ArchitectureDocumentLatex.cpp
 *
 *  Created on: Sep 14, 2017
 *      Author: cadm
 */

#include "ArchitectureDocumentLatex.h"
#include "ViewPacket.h"

#include <iostream>

/**
 * Constructor: Saves the pointer to the Database interface class and initializes internal data.
 *
 */
ArchitectureDocumentLatex::ArchitectureDocumentLatex(DatabaseInterface *pDatabaseInterface) {
	m_pDatabaseInterface = pDatabaseInterface;

}

ArchitectureDocumentLatex::~ArchitectureDocumentLatex() {
	// TODO Auto-generated destructor stub
}

/**
 * Generate the LaTeX output and store it in the filename given.
 *
 * @param sFilename - The output file.
 * @return 0 ok.
 */
int ArchitectureDocumentLatex::GenerateDocument(std::string sFilename) {
	int nStatus = 0;

	// Open the output file
	std::ofstream cFile;
	cFile.open(sFilename.c_str()); // TODO V Add error handling.

	GenerateDocumentLeadIn(&cFile);

	GenerateTitlePage(&cFile);

	GenerateContentListPages(&cFile);

	GenerateIntroductionChapter(&cFile);

	GenerateSystemOverview(&cFile);
	GenerateMappingBetweenViews(&cFile);
	GenerateDirectory(&cFile);
	GenerateRationaleBackgroundDesignConstraints(&cFile);

	cFile << "\\part{Volume II software architecture views}" << std::endl;
	cFile << "" << std::endl;
	cFile << "% TODO V make the view chapters dynamic, so that their selection is based on what is actually in the xml." << std::endl;
	cFile << "" << std::endl;

	GenerateModulePart(&cFile);
	GenerateComponentAndConnectorsPart();
	GenerateAllocationPart();
	GenerateAppendix();

	GenerateDocumentLeadOut(&cFile);
	cFile.close();

	return (nStatus);
}

/**
 * Generates the LaTeX code for the start of the document.
 * Closed with GenerateDocumentLeadOut()
 */
int ArchitectureDocumentLatex::GenerateDocumentLeadIn(std::ofstream *pFile) {
	int nStatus = 0;

	*pFile << "%% itp2doc.pl created this file." << std::endl;

	*pFile << "% https://en.wikibooks.org/wiki/LaTeX/Page_Layout" << std::endl;
	*pFile << "\\documentclass[a4paper,english]{book}" << std::endl;
	*pFile << "" << std::endl;
	*pFile
			<< "% TODO V Consider using TWO-coloumn, to improve use of paper, and keep readabiliy:"
			<< std::endl;
	*pFile
			<< "%           https://en.wikibooks.org/wiki/LaTeX/Page_Layout#Margins"
			<< std::endl;
	*pFile << "\\usepackage[cm]{fullpage}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "" << std::endl;
	*pFile
			<< "% http://tex.stackexchange.com/questions/108300/latex-split-long-table-in-multiple-pages-and-resize-the-width"
			<< std::endl;
	*pFile << "% https://en.wikibooks.org/wiki/LaTeX/Tables" << std::endl;
	*pFile << "\\usepackage{longtable}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\usepackage[T1]{fontenc}" << std::endl;
	*pFile << "\\usepackage[latin9]{inputenc}" << std::endl;
	*pFile << "\\usepackage{babel}" << std::endl;
	*pFile << "\\usepackage{graphicx}" << std::endl;
	*pFile << "" << std::endl;
	*pFile
			<< "% Provide a tool the should prevent float?: Place this after the table/figure you dont want to float past this point. \\FloatBarrier"
			<< std::endl;
	*pFile
			<< "%  http://tex.stackexchange.com/questions/2275/keeping-tables-figures-close-to-where-they-are-mentioned"
			<< std::endl;
	*pFile << "%  http://ctan.org/pkg/placeins" << std::endl;
	*pFile << "\\usepackage{placeins}" << std::endl;
	*pFile << "" << std::endl;
	*pFile
			<< "% https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=1&cad=rja&uact=8&ved=0ahUKEwj6qbq2wPrPAhWKNo8KHZYcD5oQFggcMAA&url=https%3A%2F%2Fen.wikibooks.org%2Fwiki%2FLaTeX%2FHyperlinks&usg=AFQjCNGh4ogmvoX0hclM3iuFWM3Pd9hPpA&sig2=H4gSpTFl1ewflIs8xCIJpw"
			<< std::endl;
	*pFile << "\\usepackage{hyperref}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\begin{document}" << std::endl;


	return (nStatus);
}


/**
 * Generates the LaTeX title page.
 *
 */
int ArchitectureDocumentLatex::GenerateTitlePage(std::ofstream *pFile) {
	int nStatus = 0;

	std::string sTitle = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Title");
	std::string sRevision = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Revision");
	std::string sIssue = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Issue");
	std::string sReleaseDate = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "ReleaseDate");

	// TODO V this can have multiple entries.
	std::string sAuthor = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Author");

	*pFile << "%% ====== TITLE PAGE" << std::endl;
	*pFile
			<< "% TODO V Change this to a \\maketitle environment so that I can include graphics etc."
			<< std::endl;
	*pFile << "\\title{" << sTitle << "}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "% TODO V Support multiple authors." << std::endl;
	*pFile << "\\author{ " << sAuthor << "}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\date{{ " << sReleaseDate << "}\\\\" << std::endl;
	*pFile << "Issue: { " << sIssue << "}" << std::endl;
	*pFile << "}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "% TODO V Add Summary." << std::endl;
	*pFile << "\\maketitle" << std::endl;
	*pFile << "\\pagebreak{}" << std::endl;


	return (nStatus);
}

/**
 * Generates the Table of content, list of figures, and list of tables
 */
int ArchitectureDocumentLatex::GenerateContentListPages(std::ofstream *pFile) {
	int nStatus = 0;
	*pFile << "\\tableofcontents{}" << std::endl;
	*pFile << "\\pagebreak{}" << std::endl;
	*pFile << "\\listoffigures" << std::endl;
	*pFile << "\\pagebreak{}" << std::endl;
	*pFile << "\\listoftables" << std::endl;
	*pFile << "\\pagebreak{}" << std::endl;

	return (nStatus);
}


/**
 * Generate the Introduction chapter and set the \part{}
 */
int ArchitectureDocumentLatex::GenerateIntroductionChapter(std::ofstream *pFile) {
	int nStatus = 0;

	// TODO N Make the '1' a variable instead.
	std::string sTitle = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Title");

	*pFile
			<< "\\part{ " << sTitle << " Software architecture Documentation beyond views}"
			<< std::endl;
	*pFile << "\\chapter{Introduction}" << std::endl;
	*pFile << "\\section{Purpose}" << std::endl;
	*pFile << "\\section{Scope}" << std::endl;
	*pFile << "\\section{References}" << std::endl;
	*pFile << "\\section{Acronyms}" << std::endl;
	*pFile << "\\subsection{Notations used in this document}" << std::endl;

	// Arc doc roadmap
	*pFile << "\\section{Architecture document roadmap}" << std::endl;
	*pFile << "\\label{labelArchitectureDocumentRoadmap}" << std::endl;

	*pFile << "\\subsection{Description of this documentation package}" << std::endl;
	*pFile << "This section describes the structure and contents of the entire software architecture documentation package." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "This architecture documentation package is arranged in two parts:" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item Part I: contains the information that applies to more than one view, including this roadmap of the entire package." << std::endl;
	*pFile << "\\item Part II: contains the architectural views.(TODO explain what architectural views are)." << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item Section \\ref{labelArchitectureDocumentRoadmap}, Architecture document roadmap on page \\pageref{labelArchitectureDocumentRoadmap}: lists and outlines the contents of the" << std::endl;
	*pFile << "overall documentation package and explains how stakeholder concerns can be addressed by" << std::endl;
	*pFile << "the individual parts. This is the first document that a new stakeholder should read." << std::endl;
	*pFile << "\\end{itemize}" << std::endl;

	*pFile << "\\subsection{How stakeholders can use this document}" << std::endl;
	*pFile << "" << std::endl;

	*pFile << "\\paragraph*{Someone new to the project}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "Read the documentation roadmap for an understanding of the documentation" << std::endl;
	*pFile << "package and the view template to understand how views are documented. " << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item system overview and system-level design rationale. " << std::endl;
	*pFile << "\\item top-level view packets of the " << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item module decomposition view." << std::endl;
	*pFile << "\\item pipe-and-filter view. " << std::endl;
	*pFile << "\\item deployment view. " << std::endl;
	*pFile << "\\item work assignment view." << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph*{Security analyst}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "Read the deployment view to understand the physical environment in" << std::endl;
	*pFile << "which the system operates. " << std::endl;


	*pFile << "\\paragraph*{Maintainer}" << std::endl;

	*pFile << "Read the system-level rationale in Volume I. decomposition view to" << std::endl;
	*pFile << "understand the units of implementation that exist and the area of" << std::endl;
	*pFile << "responsibility of each." << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item generalization view to see how the units relate to one another in" << std::endl;
	*pFile << "terms of generalization and specialization." << std::endl;
	*pFile << "\\item Especially read the rationale in each view and in each interface specification." << std::endl;
	*pFile << "\\item top-level view packet of the deployment view to understand where each" << std::endl;
	*pFile << "software unit is allocated." << std::endl;
	*pFile << "\\item implementation view to understand how the code units are allocated" << std::endl;
	*pFile << "to the development environment." << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph*{Developer}" << std::endl;

	*pFile << "Read the" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item decomposition view to understand the basic units of software in the system," << std::endl;
	*pFile << "\\item uses view to see how the current subset being developed is structured," << std::endl;
	*pFile << "\\item implementation view to see where the software resides in the development environment," << std::endl;
	*pFile << "\\item layered view to see which software developers are allowed to use" << std::endl;
	*pFile << "\\item work assignment view to understand the other organizational units" << std::endl;
	*pFile << "with which developers must coordinate." << std::endl;
	*pFile << "\\item interface specifications of other units to find out how to use them." << std::endl;
	*pFile << "\\item Then, using the mapping between views, read the relevant parts of" << std::endl;
	*pFile << "the other views that reveal how their units are deployed onto hardware" << std::endl;
	*pFile << "or manifested as runtime components." << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph*{Project manager}" << std::endl;
	*pFile << "" << std::endl;

	*pFile << "\\paragraph*{Performance engineer}" << std::endl;
	*pFile << "" << std::endl;

	*pFile << "\\paragraph*{Customer/Acquirer}" << std::endl;

	// SwArc view template
	*pFile << "\\section{Software Architecture view template}" << std::endl;
	*pFile << "\\subsection{How the view documentation is structured}" << std::endl;

	*pFile << "Each view is presented as a number of related view packets." << std::endl;

	*pFile << "A view packet is a small, relatively self-contained bundle of information about the system or a particular part of the system, rendered" << std::endl;
	*pFile << "in the language-element and relation types-of the view to which it belongs." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "Two view packets are related to each other as either" << std::endl;
	*pFile << "\\begin{itemize}" << std::endl;
	*pFile << "\\item parent/child: because one shows a refinement of the information in the other" << std::endl;
	*pFile << "\\item siblings: because both are children of another view packet." << std::endl;
	*pFile << "\\end{itemize}" << std::endl;
	*pFile << "" << std::endl;

	*pFile << "Each view packet has the following subsections:" << std::endl;
	*pFile << "\\paragraph{Primary presentation} that shows the elements and their relationships that populate the view" << std::endl;
	*pFile << "packet." << std::endl;

	*pFile << "The primary presentation contains the information important to convey about the system," << std::endl;
	*pFile << "in the vocabulary of that view, first." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "It includes the primary elements and relations of the" << std::endl;
	*pFile << "view packet, but under some circumstances might not include all of them." << std::endl;
	*pFile << "" << std::endl;
	*pFile << " For example, the" << std::endl;
	*pFile << "primary presentation may show the elements and relations that come into play during normal" << std::endl;
	*pFile << "operation, relegating error handling or exception processing to the supporting documentation." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "The primary presentation is usually graphical. If so, the presentation will include a key that" << std::endl;
	*pFile << "explains the meaning of every symbol used." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "The first part of the key identifies the notation: If" << std::endl;
	*pFile << "a defined notation is being used, the key will name it and cite the document that defines it or" << std::endl;
	*pFile << "defines the version of it being used. If the notation is informal, the key will say so and proceed" << std::endl;
	*pFile << "to define the symbology and the meaning, if any, of colors, position, or other informationcarrying" << std::endl;
	*pFile << "aspects of the diagram." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph{Element catalog} detailing at least those elements depicted in the primary presentation and" << std::endl;
	*pFile << "others that were omitted from the primary presentation." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "Specific parts of the catalog include" << std::endl;
	*pFile << "\\subparagraph{Elements and their properties} This section names each element in the view packet and" << std::endl;
	*pFile << "lists the properties of that element." << std::endl;
	*pFile << "" << std::endl;
	*pFile << " For example, elements in a module decomposition" << std::endl;
	*pFile << "view have the property of \"responsibility,\" an explanation of each module's role in the" << std::endl;
	*pFile << "system, and elements in a communicating-process view have timing parameters, among" << std::endl;
	*pFile << "other things, as properties." << std::endl;
	*pFile << "\\subparagraph{Relations and their properties} Each view has a specific type of relation that it depicts" << std::endl;
	*pFile << "among the elements in that view. However, if the primary presentation does not show" << std::endl;
	*pFile << "all the relations or if there are exceptions to what is depicted in the primary presentation," << std::endl;
	*pFile << "this section will record that information." << std::endl;
	*pFile << "\\subparagraph{Element interface} An interface is a boundary across which elements interact or communicate" << std::endl;
	*pFile << "with each other." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "This section is where element interfaces are documented." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subparagraph{Element behavior} Some elements have complex interactions with their environment" << std::endl;
	*pFile << "and for purposes of understanding or analysis, the element's behavior is documented." << std::endl;
	*pFile << "The behavior of elements is specified in component-and-connector views in (TODO C Provide the actual references here:)Volume" << std::endl;
	*pFile << "II, Chapters 5, 6, and 7." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph{Context diagram} showing how the system depicted in the view packet relates to its environment." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph{Variability guide} showing how to exercise any variation points that are a part of the architecture" << std::endl;
	*pFile << "shown in this view packet." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph{Architecture background} explaining why the design reflected in the view packet came to be." << std::endl;
	*pFile << "The goal of this section is to explain why the design is as it is and to provide a convincing" << std::endl;
	*pFile << "argument that it is sound. Architecture background includes" << std::endl;
	*pFile << "\\subparagraph{Design rationale} This explains why the design decisions reflected in the view packet were" << std::endl;
	*pFile << "made and gives a list of rejected alternatives and why they were rejected. This will prevent" << std::endl;
	*pFile << "future architects from pursuing dead ends in the face of required changes." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subparagraph{Analysis results} This documents the results of analyses that have been conducted," << std::endl;
	*pFile << "such as the results of performance or security analyses, or a list of what would have to" << std::endl;
	*pFile << "change in the face of a particular kind of system modification." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subparagraph{Assumptions} This documents any assumptions the architect made when crafting the design." << std::endl;

	*pFile << " Assumptions are generally about either environment or need." << std::endl;
	*pFile << "" << std::endl;

	*pFile << "Environmental assumptions document what the architect assumes is available in the" << std::endl;
	*pFile << "environment that can be used by the system being designed. They also include assumptions" << std::endl;
	*pFile << "about invariants in the environment." << std::endl;
	*pFile << "" << std::endl;
	*pFile << " For example, a navigation system architect" << std::endl;
	*pFile << "might make assumptions about the stability of Earth's geographic and/or magnetic poles." << std::endl;
	*pFile << "" << std::endl;
	*pFile << " Finally, assumptions about the environment may be about the development" << std::endl;
	*pFile << " environment: tool suites available or the skill levels of the implementation teams, for example." << std::endl;
	*pFile << " " << std::endl;
	*pFile << "Assumptions about need state why the design provided is sufficient for what's needed." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "For example, if a navigation system's software interface provides location information in" << std::endl;
	*pFile << "a single geographic frame of reference, the architect is assuming that is sufficient, and" << std::endl;
	*pFile << "that alternative frames of reference are not useful." << std::endl;
	*pFile << "" << std::endl;

	*pFile << "\\paragraph{Other information} This section includes nonarchitectural and organization-specific information." << std::endl;
	*pFile << "will usually include" << std::endl;
	*pFile << "management or configuration control information, change histories, bibliographic references or lists of useful companion documents," << std::endl;
	*pFile << "mapping to requirements, and the like." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph{Related view packets} This section will name other view packets that are related to the one" << std::endl;
	*pFile << "being described in a parent/child or sibling capacity." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection{UML notation used in this document}" << std::endl;
	*pFile << "TODO C Also include a description of the UML used in this document." << std::endl;

	return (nStatus);
}


int ArchitectureDocumentLatex::GenerateSystemOverview(std::ofstream *pFile) {
	int nStatus = 0;

	std::string sBackground = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Background");
	std::string sWhatWillNotBeDone = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "WhatWillNotBeDone");
	std::string sPrototypeDeployment = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "PrototypeDeployment");
	std::string sProductionDeployment = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "ProductionDeployment");
	std::string sPlannedUpgrades = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "PlannedUpgrades");
	std::string sIntendedOperationalLife = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "IntendedOperationalLife");


	*pFile << "\\chapter{System overview}" << std::endl;
	*pFile << "\\section{Overview}" << std::endl;
	*pFile << "\\subsection{Background}" << std::endl;
	*pFile << sBackground << std::endl;
	*pFile << "\\subsection{Broad requirements}" << std::endl;

	// TODO C Add support for BroadRequirements in  SoftwareArchitectureDocumentation
	/*
	*pFile << "{ foreach my $szUuid (keys %{$Project{'hBroadRequirements'}}) {" << std::endl;
	*pFile << "    $OUT .= "\\paragraph{$szUuid - $Project{hBroadRequirements}{$szUuid}}\n";" << std::endl;
	*pFile << "  }" << std::endl;
	*pFile << "}" << std::endl;
	*/

	*pFile << "\\subsection{What will not be done}" << std::endl;
	*pFile << sWhatWillNotBeDone << std::endl;
	*pFile << "" << std::endl;

	*pFile << "\\subsection{Deploying the product}" << std::endl;


	*pFile << "\\subsubsection{Prototype deployment}" << std::endl;
	*pFile << sPrototypeDeployment << std::endl;

	*pFile << "\\subsubsection{Production deployment}" << std::endl;
	*pFile << sProductionDeployment << std::endl;

	*pFile << "\\subsubsection{Planned upgrades}" << std::endl;
	*pFile << sPlannedUpgrades << std::endl;

	*pFile << "\\subsubsection{Intended operational life}" << std::endl;
	*pFile << sIntendedOperationalLife << std::endl;


	*pFile << "\\subsection{System level context diagram}" << std::endl;
	// TODO V Add support for the ContextModel
	//*pFile << "\\input{{$Project{'ContextModelFileName'}}}" << std::endl;

	*pFile << "\\subsection{Physical entities}" << std::endl;
	*pFile << "%TODO V Support this when needed." << std::endl;
	*pFile << "\\subsection{FAQ}" << std::endl;
	*pFile << "%TODO N it would be nice to have it here." << std::endl;

	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateMappingBetweenViews(std::ofstream *pFile) {
	int nStatus = 0;
	*pFile << "\\chapter{Mapping between views}" << std::endl;
	*pFile << "This chapter establishes the relationship between corresponding elements and structures that appear" << std::endl;
	*pFile << "in different views in part (TODO V Provide reference to the correct part). Not all pairwise view combinations have a mapping in this" << std::endl;
	*pFile << "section, and not every element is mapped. The emphasis is on mappings that provide useful insights." << std::endl;
	*pFile << "The following table summarizes the mappings in this chapter. The table cells tell which section" << std::endl;
	*pFile << "contains a mapping between the corresponding views on the cell's row and column header." << std::endl;

	*pFile << "\\section{Excerpts from a Software Architecture Documentation Package}" << std::endl;

	*pFile << "\\section{Mapping Between Module Decomposition View and Module Generalization View}" << std::endl;

	*pFile << "\\section{Mapping Between Module Decomposition View and C\\&C Shared-Data View}" << std::endl;

	*pFile << "\\section{Mapping Between Module Decomposition View and C\\&C Communicating-Processes view}" << std::endl;


	*pFile << "\\section{Mapping Between Module Decomposition View, Module Uses View, Allocation Implementation View, and Allocation Work Assignment View}" << std::endl;

	*pFile << "TODO is this always valid:" << std::endl;
	*pFile << "{\\em The segments and subsystems named in the two allocation views and the module uses view are" << std::endl;
	*pFile << "exactly the same elements as those named in the module decomposition view.}" << std::endl;


	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateDirectory(std::ofstream *pFile) {
	int nStatus = 0;
	*pFile << "\\chapter{Directory}" << std::endl;
	*pFile << "	The directory is an index of all the elements, relations, and properties that appear in any of the views." << std::endl;
	*pFile << "	This index is intended to help a reader find all places where a particular kind of element, relation," << std::endl;
	*pFile << "	or property is used." << std::endl;

	*pFile << "	\\begin{table}[!htbp]" << std::endl;
	*pFile << "	  \\begin{tabular}{|l|l|l|} \\hline" << std::endl;
	*pFile << "	Item & Type & Reference \\\\ \\hline" << std::endl;
	*pFile << "	  \\end{tabular}" << std::endl;
	*pFile << "	  \\caption{Context model names}" << std::endl;
	*pFile << "	\\end{table}" << std::endl;

	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateRationaleBackgroundDesignConstraints(std::ofstream *pFile) {
	int nStatus = 0;
	*pFile << "\\chapter{Rationale, Background, and Design constraints}" << std::endl;
	*pFile << "The following records those contextual and requirements aspects that were the primary" << std::endl;
	*pFile << "motivators behind the major architectural decisions." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "There were, of course, a large number of specific" << std::endl;
	*pFile << "requirements that had to be satisfied, but these are the major ones that had profound architectural" << std::endl;
	*pFile << "influence." << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\subsection{Why the architecture is the way it is}" << std::endl;

	*pFile << "\\paragraph{Business context}" << std::endl;

	*pFile << "\\paragraph{SIPOC}" << std::endl;
	*pFile << "TODO SUPPORT SIPOC" << std::endl;
	// TODO N Support for SystemLevelSipoc
	// *pFile << "{ $Project{"SystemLevelSipoc"} }" << std::endl;
	*pFile << "" << std::endl;
	*pFile << "\\paragraph{Key data management features}" << std::endl;
	*pFile << "\\paragraph{Key data ingest features}" << std::endl;
	*pFile << "\\paragraph{Key data processing features}" << std::endl;
	*pFile << "\\paragraph{Key user interface features}" << std::endl;
	*pFile << "" << std::endl;

	return (nStatus);
}

/**
 * Generate the
 */
int ArchitectureDocumentLatex::GenerateModulePart(std::ofstream *pFile) {
	int nStatus = 0;


	// Get list of ModuleDecompositionViewPacketId
	nStatus = GenerateModuleDecomposition(pFile);

	// TODO C Get list of ModuleUsesViewPacketId
	// TODO C Get list of ModuleGeneralizationViewPacketId
	// TODO C Get list of ModuleLayeredViewPacketId

	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateModuleDecomposition(std::ofstream *pFile) {
	int nStatus = 0;
	std::list<std::string> lViewPacketIdList = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationViewPacketIdList(1, "Module", "Decomposition");
	 for (std::list<std::string>::iterator itPacketId=lViewPacketIdList.begin(); itPacketId != lViewPacketIdList.end(); ++itPacketId) {
		 std::cout << "DDD GenerateModuleDecomposition idx: " << (*itPacketId) << std::endl;
		 ViewPacket *pViewPacket = new ViewPacket(m_pDatabaseInterface, *itPacketId);
		 pViewPacket->SaveSection(pFile);
		 delete pViewPacket;
	 }


	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateModuleUses(std::ofstream *pFile) {
	int nStatus = 0;
	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateModuleGeneralization(std::ofstream *pFile) {
	int nStatus = 0;
	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateModuleLayered(std::ofstream *pFile) {
	int nStatus = 0;
	return (nStatus);
}

int ArchitectureDocumentLatex::GenerateComponentAndConnectorsPart() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateAllocationPart() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateAppendix() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateDocumentLeadOut(std::ofstream *pFile) {
	int nStatus = 0;

	*pFile << "\\end{document}" << std::endl;

	return (nStatus);
}
