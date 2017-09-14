/*
 * DatabaseInterface.h
 *
 *  Created on: Sep 14, 2017
 *      Author: cadm
 */

#ifndef DATABASEINTERFACE_H_
#define DATABASEINTERFACE_H_

#include <string>

/**
 * This is the interface to the data (Currently the XML file).
 *
 */
class DatabaseInterface {
public:
	DatabaseInterface(std::string);
	virtual ~DatabaseInterface();
};

#endif /* DATABASEINTERFACE_H_ */
