/*
 * XmlSupport.cpp
 *
 *  Created on: May 29, 2017
 *      Author: cadm
 */

#include "XmlSupport.h"

XmlSupport::XmlSupport() {
	// TODO Auto-generated constructor stub

}

XmlSupport::~XmlSupport() {
	// TODO Auto-generated destructor stub
}

/**
 * Retrieve the Value of the Attribute on the given element.
 *
 * @param xmlElement - The element on which to retrieve the content of attribute of xmlElement
 * @param sAttributeName - Name of the attribute
 * @return Value of the given attribute.
 */
std::string XmlSupport::GetAttributeData(tinyxml2::XMLElement* xmlElement, std::string sAttributeName) {
	std::string sResult = "";

	if (xmlElement != NULL) {
		if (xmlElement->Attribute(sAttributeName.c_str())) {
			sResult = xmlElement->Attribute(sAttributeName.c_str());
		}
	}
	return (sResult);
}

/**
 * Return The text content the sTagName sub-element of xmlElement.
 *
 * @param xmlElement - XML Element.
 * @param sTagName - Name of sub-element.
 * @return The text content the sTagName sub-element.
 */
std::string XmlSupport::GetChildDataBySingleTagName(tinyxml2::XMLElement* xmlElement,
		std::string sTagName) {
	std::string sResult = "";
	tinyxml2::XMLElement *xmlChildElement = NULL;

	if (xmlElement != NULL) {
		xmlChildElement = xmlElement->FirstChildElement(sTagName.c_str());
	}
	// From https://stackoverflow.com/questions/7942191/how-to-handle-tinyxml-null-pointer-returned-on-gettext
	if ( (xmlChildElement != NULL) && (xmlChildElement->GetText() != NULL)){
		sResult = xmlChildElement->GetText();
	}
	return (sResult);
}
