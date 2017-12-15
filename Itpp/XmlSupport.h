/*
 * XmlSupport.h
 *
 *  Created on: May 29, 2017
 *      Author: cadm
 */

#ifndef XMLSUPPORT_H_
#define XMLSUPPORT_H_

#include <string>
#include <tinyxml2.h>
#include <list>

/**
 * The XmlSupport class provides functions to manipulate XML files/structures.
 *
 * Example:
 *   XmlSupport m_cXmlSupport;
 *   std::string sText = m_cXmlSupport.GetChildDataBySingleTagName(m_xmlRoot, "TextData");
 */
class XmlSupport {
public:
	XmlSupport();
	virtual ~XmlSupport();

	std::string GetAttributeData(tinyxml2::XMLElement* , std::string);
	std::string GetChildDataBySingleTagName(tinyxml2::XMLElement* , std::string);
	tinyxml2::XMLElement* GetSingleChildNodeByTagAndAttribute(tinyxml2::XMLElement*, std::string, std::string, std::string );
	//tinyxml2::XMLElement* GetNodeArrayByTagAndAttribute(tinyxml2::XMLElement*, std::string, std::string, std::string );

	std::list<std::string> GetListOfChildData(tinyxml2::XMLElement*,std::string);
	std::list<std::string> GetListOfChildDataSortedByAttribute(tinyxml2::XMLElement*,std::string,std::string);
};

#endif /* XMLSUPPORT_H_ */
