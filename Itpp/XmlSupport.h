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

/**
 * The XmlSupport class provides functions to manipulate XML files/structures.
 */
class XmlSupport {
public:
	XmlSupport();
	virtual ~XmlSupport();

	std::string GetAttributeData(tinyxml2::XMLElement* , std::string);
	std::string GetChildDataBySingleTagName(tinyxml2::XMLElement* , std::string);
	tinyxml2::XMLElement* GetSingleChildNodeByTagAndAttribute(tinyxml2::XMLElement*, std::string, std::string, std::string );
	//tinyxml2::XMLElement* GetNodeArrayByTagAndAttribute(tinyxml2::XMLElement*, std::string, std::string, std::string );

};

#endif /* XMLSUPPORT_H_ */
