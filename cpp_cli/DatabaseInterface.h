/*
 * DatabaseInterface.h
 *
 *  Created on: Sep 14, 2017
 *      Author: cadm
 */

#ifndef DATABASEINTERFACE_H_
#define DATABASEINTERFACE_H_

#include <string>
#include "XmlSupport.h"

#include "ComponentRelationLists.h"



/**
 * This is the interface to the data (Currently the XML file).
 *
 */
class DatabaseInterface {
public:
	DatabaseInterface(std::string);
	virtual ~DatabaseInterface();

	ComponentRelationLists* GetComponentTree(std::string, int);
	std::string GetSoftwareArchitectureDocumentationElementText(int, std::string);
	std::list<std::string> GetSoftwareArchitectureDocumentationViewPacketIdList(int, std::string, std::string);
	std::string GetViewPacketElementText(std::string, std::string);

private:
	tinyxml2::XMLDocument m_xmlDoc; /// The actual XML document.
	tinyxml2::XMLElement* m_xmlRoot; /// The actual root of the XML document.

	XmlSupport m_cXmlSupport;
};

#endif /* DATABASEINTERFACE_H_ */
