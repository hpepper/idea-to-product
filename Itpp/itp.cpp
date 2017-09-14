//============================================================================
// Name        : itp.cpp
// Author      : Author
// Version     :
// Copyright   : Your copyright notice
//============================================================================

// Example: ./itp

/*! \mainpage Idea to Product - SW development and documentation
 *
 * \section intro_sec Introduction
 *
 * generate the documentation from the xml file.
 *
 * \section overview_sec Overview
 *
 *
 * \section install_sec Installation
 *
 * \subsection step1 Step 1: Opening the box
 *
 * \section Design
 *
 *
 *
 * etc...
 */

#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <iostream>

#include <fstream>

/**
 * Main Function.
 *
 * This is the starting point.
 * The ToolBox.cpp uses the CharacterSheet and the SheetLayoutLoader.
 *
 * @see CharacterSheet
 * @see SheetLayoutLoader
 */
int main(int argc, char* argv[]) {
	puts("RPG ToolBox");

	/*
	 f_cConfigurationsPaths.setLogosPath("../logos/");
	 f_cConfigurationsPaths.setSheetDefinitionPath("cfg/");
	 f_cConfigurationsPaths.setToolBoxConfigurationFilesPath("./");
	 */
	if (argc < 3) {
		std::cerr << "Usage: " << argv[0] << " COMMAND XML_FILE" << std::endl;
		std::cerr << "  COMMAND:" << std::endl;
		std::cerr
				<< "    convert Save a new XML with the CharacterSheet converted to the target system."
				<< std::endl;
		std::cerr << "    help    Get help on command." << std::endl;
		std::cerr << "    ps      Generate post script character sheet."
				<< std::endl;
		std::cerr
				<< "  XML_FILE: character sheet xml file. The file to operate on."
				<< std::endl;
		return (-1);
	}
	std::string sCharacterDataFileName;
	// TODO Get convert
	std::string sCommand = argv[1];
	// verify that this is a valid command.
	// TODO Find a more fancy way of testing this, lists or something else?

	return EXIT_SUCCESS;
}

