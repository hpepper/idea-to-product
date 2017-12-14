/*
 * ComponentTree.h
 *
 *  Created on: Sep 24, 2017
 *      Author: cadm
 */

#ifndef COMPONENTRELATIONLISTS_H_
#define COMPONENTRELATIONLISTS_H_

#include <string>
#include <list>

/**
 * The ComponentTree class holds the information needed to generate a graphviz tree and legend table.
 *
 * Graphviz needs a list of entities(components) and a information on how the entities are connected.
 *
 * The legend needs
 *   - a list of Component IDs (then the user can look up the details like name and descriptions.
 *   - a list of the relation IDs.
 *
 * @see DatabaseInterface for how it is populated.
 * @see ViewPacket for how it is being used.
 */
class ComponentRelationLists {
public:
	ComponentRelationLists();
	virtual ~ComponentRelationLists();

	int PushComponentId(int nComponentId) { m_cListOfComponents.push_back(nComponentId); return(m_cListOfComponents.size()); }
	int PushComponentRelationId(int nRelationId) { m_cListOfComponentRelations.push_back(nRelationId); return(m_cListOfComponentRelations.size());}

	std::list<int> GetRomponentList() { return(m_cListOfComponents); }
	std::list<int> GetRomponentRelationsList() { return(m_cListOfComponentRelations); }

private:
	std::list<int> m_cListOfComponents;
	std::list<int> m_cListOfComponentRelations;
};

#endif /* COMPONENTRELATIONLISTS_H_ */
