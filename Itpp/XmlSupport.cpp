/*
 * XmlSupport.cpp
 *
 *  Created on: May 29, 2017
 *      Author: cadm
 */

#include "XmlSupport.h"

#include <assert.h>

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
 * Return the XMLElement pointer to the first XML element where the sAttributeName has the value: sAttributeContent.
 *
 * Example:
 *   tinyxml2::XMLElement* xmlElement = m_pXmlSupport->GetSingleChildNodeByTagAndAttribute(m_xmlRoot, "Alpha", "Id", "2");
 *   	if ( xmlElement !=  NULL ) {
 *   		  sTitle = xmlElement->Attribute("Title");
 *   		  	}
 *
 */
tinyxml2::XMLElement* XmlSupport::GetSingleChildNodeByTagAndAttribute(
		tinyxml2::XMLElement *xmlParentElement, std::string sTagName,
		std::string sAttributeName, std::string sAttributeContent) {

	tinyxml2::XMLElement* xmlReturnXmlElement = NULL;

	assert(xmlParentElement != NULL);

	tinyxml2::XMLElement* xmlCurrentElement =
			xmlParentElement->FirstChildElement(sTagName.c_str());

	// Itterate until the right attribute is found, or there are no more elements.
	while ((xmlCurrentElement != NULL) && (xmlReturnXmlElement == NULL)) {
		int nDummy;

		if ( xmlCurrentElement->QueryAttribute(sAttributeName.c_str(), &nDummy) != tinyxml2::XML_NO_ATTRIBUTE) {
			const char* szCurrentAttributeValue = xmlCurrentElement->Attribute( sAttributeName.c_str() );
			if ( sAttributeContent.compare(szCurrentAttributeValue) == 0 ) {
				xmlReturnXmlElement = xmlCurrentElement;
			}
		}
		xmlCurrentElement = xmlCurrentElement->NextSiblingElement(sTagName.c_str());
	}

	// Find the one with the Attribute with the sAttributeContent
	return (xmlReturnXmlElement);
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
