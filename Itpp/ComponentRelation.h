/*
 * ComponentRelation.h
 *
 *  Created on: Sep 17, 2017
 *      Author: cadm
 */

#ifndef COMPONENTRELATION_H_
#define COMPONENTRELATION_H_

#include "Component.h"

#include <string>

typedef std::list<ComponentRelation*> COMPONENT_RELATION_POINTER_LIST;

/**
 * Sub component it is related to
 * and relation type.
 *
 * Then later generate the grapviz.
 */
class ComponentRelation {
public:
	ComponentRelation(Component*, std::string);
	virtual ~ComponentRelation();

private:
	// TODO V Some how I need to sure that all relations
	Component *m_pComponent;
	std::string m_sRelationType;
};

#endif /* COMPONENTRELATION_H_ */
