#include "DatabaseInterface.h"

#include <cxxtest/TestSuite.h>
#include <assert.h>


class DatabaseInterfaceTestSuite: public CxxTest::TestSuite {

	DatabaseInterface *m_pDatabaseInterface;
	tinyxml2::XMLDocument m_xmlDoc;
	tinyxml2::XMLElement *m_xmlRoot;

public:

	void setUp() {
		m_pDatabaseInterface = new DatabaseInterface("unittest_databaseinterface.xml");
	}

	void tearDown() {

		delete m_pDatabaseInterface;
	}

	void testGetSoftwareArchitectureDocumentationElementTextTitle() {
		std::string sTitle = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "Title");
		TS_ASSERT_EQUALS(sTitle, "First Title");
	}

	void testGetSoftwareArchitectureDocumentationElementSecondTitle() {
		std::string sTitle = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(2, "Title");
		TS_ASSERT_EQUALS(sTitle, "Second title");
	}
	void testGetSoftwareArchitectureDocumentationElementTextNotExist() {
		std::string sTitle = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationElementText(1, "DOES_NOT_EXIST");
		TS_ASSERT_EQUALS(sTitle, "");
	}

	void testGetSoftwareArchitectureDocumentationViewPacketIdList() {
		std::list<std::string> lViewPacketIdList = m_pDatabaseInterface->GetSoftwareArchitectureDocumentationViewPacketIdList(1, "Module", "Decomposition");
		TS_ASSERT_EQUALS(lViewPacketIdList.size(), 2);
	}



	// TODO V Test with XML file not exist.
};
