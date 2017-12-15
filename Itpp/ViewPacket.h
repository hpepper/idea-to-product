/*
 * ViewPacket.h
 *
 *  Created on: Sep 16, 2017
 *      Author: cadm
 */

#ifndef VIEWPACKET_H_
#define VIEWPACKET_H_

#include "DatabaseInterface.h"
#include <string>
#include <iostream>
#include <fstream>

class ViewPacket {
public:
	ViewPacket(DatabaseInterface*, std::string);
	virtual ~ViewPacket();

	int SaveSection(std::ofstream*);

	std::string GetTitle();

private:
	DatabaseInterface *m_pDatabaseInterface;
	std::string m_sViewPacketIndex;

};

#endif /* VIEWPACKET_H_ */
