#include "ComponentRelationLists.h"

#include <cxxtest/TestSuite.h>
#include <assert.h>



class ComponentRelationListsTestSuite: public CxxTest::TestSuite {

	ComponentRelationLists m_cComponentRelationLists;

public:

	void setUp() {
	}

	void tearDown() {
	}

	void testPushComponentId() {
		int nStatus = m_cComponentRelationLists.PushComponentId(1);
		TS_ASSERT_EQUALS(nStatus, 1);
	}
};
