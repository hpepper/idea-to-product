/*
 * ArchitectureDocumentLatex.h
 *
 *  Created on: Sep 14, 2017
 *      Author: cadm
 */

#ifndef ARCHITECTUREDOCUMENTLATEX_H_
#define ARCHITECTUREDOCUMENTLATEX_H_

#include "ArchitectureDocument.h"

#include "DatabaseInterface.h"

#include <string>

#include <fstream>

/**
 * Generate the Latex version of the SW Architecture document.
 *
 */
class ArchitectureDocumentLatex: public ArchitectureDocument {
public:
	ArchitectureDocumentLatex(DatabaseInterface *);
	virtual ~ArchitectureDocumentLatex();

	int GenerateDocument(std::string);

private:

	int GenerateDocumentLeadIn(std::ofstream *);
	int GenerateDocumentLeadOut(std::ofstream *);

	int GenerateTitlePage(std::ofstream *);

	int GenerateIntroductionChapter();

	int GenerateSystemOverview();
	int GenerateMappingBetweenViews();
	int GenerateDirectory();
	int GenerateRationaleBackgroundDesignConstraints();
	int GenerateModulePart();
	int GenerateCAndConnectorsPart();
	int GenerateAllocationPart();
	int GenerateAppendix();

	DatabaseInterface *m_pDatabaseInterface; /// Holds the pointer to the DatabaseInterface class.
};

#endif /* ARCHITECTUREDOCUMENTLATEX_H_ */
