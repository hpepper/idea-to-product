/*
 * DatabaseInterface.cpp
 *
 */

#include "DatabaseInterface.h"

#include <string>

#include "ComponentRelationLists.h"
//#include <iostream> // cout, endl


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
 * GetComponentTree is just a proxy for calling and generating the ComponentTree();
 *
 * Get
 * List of Component id
 * List of Component link ID
 *
 * Then the caller can populate the information for a Graphviz image and Legend table.
 * TODO
 */
ComponentRelationLists *DatabaseInterface::GetComponentTree(std::string sComponentId, int nComponentRecurseLevel) {
	// TODO V Validate that sComponentId is a number.
	// TODO V validate that nComponentRecurseLevel is a positive number.
	ComponentRelationLists * pComponentRelationLists = new ComponentRelationLists();
	// ComponentRelationLists *pComponentTree = pComponentRelationLists->
	return(pComponentRelationLists);
}

/**
 * Retrieve the titel of the SoftwareArchitectureDocumentation at the given nIndex
 *
 * @param nIndex Index in the 'Id' attribute of the SoftwareArchitectureDocumentation tag.
 * @param sTagName - Name of the subtag that has the text that you are lookig for.
 * @return string; empty if not found.
 */
std::string DatabaseInterface::GetSoftwareArchitectureDocumentationElementText(int nIndex, std::string sTagName) {
	std::string sText = "";

	// Get the Propper XML element.
	std::string sIndexString = std::to_string(nIndex); // TODO Fix this, possibly missing the C++11 tag or something.
	tinyxml2::XMLElement* xmlElement = m_cXmlSupport.GetSingleChildNodeByTagAndAttribute(m_xmlRoot, "SoftwareArchitectureDocumentation", "Id", sIndexString);
	if ( xmlElement !=  NULL ) {
		sText = m_cXmlSupport.GetChildDataBySingleTagName(xmlElement, sTagName);
	}

	return(sText);
}


/**
 * Retrieve a list of viewpacket Id's for the given sViewName and View sStyleName from the SoftwareArchitectureDocumentation at the given nIndex
 *
 * @param nIndex Index in the 'Id' attribute of the SoftwareArchitectureDocumentation tag.
 * @param sViewName : Module | CnC | Allocate
 * @param sStyleName : Dependent on the ViewName given
 * @return list of ViewPacket Index numbers.
 */
std::list<std::string> DatabaseInterface::GetSoftwareArchitectureDocumentationViewPacketIdList (
		int nIndex, std::string sViewName, std::string sStyleName) {
	std::list<std::string> lDataList;
	std::string sTagName = sViewName + sStyleName + "ViewPacketId";
	// TODO V Create a private common function that returns the 'SoftwareArchitectureDocumentation' element given by nIndex.
	tinyxml2::XMLElement* xmlElement = m_cXmlSupport.GetSingleChildNodeByTagAndAttribute(m_xmlRoot, "SoftwareArchitectureDocumentation", "Id", std::to_string(nIndex));
	if ( xmlElement !=  NULL ) {
		lDataList = m_cXmlSupport.GetListOfChildData(xmlElement, sTagName);
	}


	return (lDataList);
}

/**
 * Retrieve the Text of the given Sub element for the ViewPacket with id: sIndexString
 *
 * @param sIndexString - Id og the ViewPacket.
 * @param sTagName - Name of the sub element.
 * @return "" if the sTagName wasn't found.
 */
std::string DatabaseInterface::GetViewPacketElementText(std::string sIndexString, std::string sTagName) {
	std::string sText = "";

	// Get the Propper XML element.
	tinyxml2::XMLElement* xmlElement = m_cXmlSupport.GetSingleChildNodeByTagAndAttribute(m_xmlRoot, "ViewPacket", "Id", sIndexString);
	if ( xmlElement !=  NULL ) {
		sText = m_cXmlSupport.GetChildDataBySingleTagName(xmlElement, sTagName);
	}

	return(sText);
}
