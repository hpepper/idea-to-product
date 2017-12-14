/*
 * ComponentRelation.cpp
 *
 *  Created on: Sep 17, 2017
 *      Author: cadm
 */

#include "ComponentRelation.h"

ComponentRelation::ComponentRelation(Component *pComponent, std::string sRelationType) {
	m_pComponent = pComponent;
	m_sRelationType = sRelationType;
}

ComponentRelation::~ComponentRelation() {
	// TODO Auto-generated destructor stub
}

