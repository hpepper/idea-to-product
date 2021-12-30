/*
 * Component.h
 *
 *  Created on: Sep 17, 2017
 *      Author: cadm
 */

#ifndef COMPONENT_H_
#define COMPONENT_H_

/**
 *
 */
class Component {
public:
	Component();
	virtual ~Component();

private:
	std::string m_sId;

	// TODO C Is it really here that this list must be maintained?
	COMPONENT_RELATION_POINTER_LIST m_lComponentRelationPointerList; /// List of relations
};

#endif /* COMPONENT_H_ */
