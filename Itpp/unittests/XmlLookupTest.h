#include "XmlSupport.h"

#include <cxxtest/TestSuite.h>
#include <assert.h>


class XmlSupportTestSuite: public CxxTest::TestSuite {

	XmlSupport *m_pXmlSupport;
	tinyxml2::XMLDocument m_xmlDoc;
	tinyxml2::XMLElement *m_xmlRoot;

public:

	void setUp() {

		int nStatus = m_xmlDoc.LoadFile("unittest_xml_support.xml");
		if (nStatus == 0) {
			m_xmlRoot = m_xmlDoc.RootElement();
			assert(m_xmlRoot != NULL);
		} else {
			m_xmlDoc.PrintError();
		}

	}

	void tearDown() {

		delete m_pXmlSupport;
	}

	void testGetAttributeData() {
		std::string sVersion = m_pXmlSupport->GetAttributeData(m_xmlRoot, "Version");
		TS_ASSERT_EQUALS(sVersion, "0.1.0");
	}

	void testGetChildDataBySingleTagName() {
		std::string sText = m_pXmlSupport->GetChildDataBySingleTagName(m_xmlRoot, "TextData");
		TS_ASSERT_EQUALS(sText, "text_data_content");
	}

	void testGetSingleChildNodeByTagAndAttribute() {
		tinyxml2::XMLElement* xmlElement = m_pXmlSupport->GetSingleChildNodeByTagAndAttribute(m_xmlRoot, "Alpha", "Id", "2");
		TS_ASSERT_EQUALS(xmlElement->Attribute("Title"), "Title-Id-2");
	}

};
