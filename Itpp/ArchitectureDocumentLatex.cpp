/*
 * ArchitectureDocumentLatex.cpp
 *
 *  Created on: Sep 14, 2017
 *      Author: cadm
 */

#include "ArchitectureDocumentLatex.h"


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

	GenerateIntroductionChapter();

	GenerateSystemOverview();
	GenerateMappingBetweenViews();
	GenerateDirectory();
	GenerateRationaleBackgroundDesignConstraints();
	GenerateModulePart();
	GenerateCAndConnectorsPart();
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


/*
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

int ArchitectureDocumentLatex::GenerateIntroductionChapter() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateSystemOverview() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateMappingBetweenViews() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateDirectory() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateRationaleBackgroundDesignConstraints() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateModulePart() {
	int nStatus = 0;
	return (nStatus);
}
int ArchitectureDocumentLatex::GenerateCAndConnectorsPart() {
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
