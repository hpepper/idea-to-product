/*
 * DatabaseInterface.cpp
 *
 *  Created on: Sep 14, 2017
 *      Author: cadm
 */

#include "DatabaseInterface.h"



/**
 * Constructor Loads the XML file given in sXmlFileNAme
 *
 * @param sXmlFileNAme
 */
DatabaseInterface::DatabaseInterface(std::string sXmlFileNAme) {

	int nStatus = m_xmlDoc.LoadFile(sXmlFileNAme.c_str());

	if (nStatus == 0) {
		m_xmlRoot = m_xmlDoc.RootElement();
	} else {
		m_xmlDoc.PrintError();
		exit(1);
	}
}

DatabaseInterface::~DatabaseInterface() {
	// TODO Auto-generated destructor stub
}

/**
 * Retrieve the titel of the SoftwareArchitectureDocumentation at the given nIndex
 *
 * @param nIndex Index in the 'Id' attribute.
 * @param sTagName - Name of the subtag that has the text that you are lookig for.
 * @return string; empty if not found.
 */
std::string DatabaseInterface::GetSoftwareArchitectureDocumentationElementText(int nIndex, std::string sTagName) {
	std::string sText = "";

	// Get the Propper XML element.
	tinyxml2::XMLElement* xmlElement = m_cXmlSupport.GetSingleChildNodeByTagAndAttribute(m_xmlRoot, "SoftwareArchitectureDocumentation", "Id", "1");
	if ( xmlElement !=  NULL ) {
		sText = m_cXmlSupport.GetChildDataBySingleTagName(xmlElement, sTagName);
	}

	return(sText);
}
