CFLAGS = -Wall -g -I.. -std=c++11 -fprofile-arcs -ftest-coverage 
GCVOR_FLAGS = -r .. --exclude unittests/runner.cpp\
                    --exclude unittests/GurpsDamageTableTest.h\
                    --exclude unittests/ConvertingDndToGurpsTest.h\
                    --exclude unittests/CharacterSheetSectionBaseTest.h\
                    --exclude unittests/GurpsCharacterSheetSectionTest.h\
                    --exclude unittests/GurpsArmorTableTest.h \
                    --exclude unittests/GurpsWeaponsTableTest.h \
                    --exclude unittests/DndCharacterSheetSectionTest.h\
                    --exclude unittests/CharacterSheet.h\
                    --exclude unittests/SheetLayoutLoaderTest.h\
                    --exclude unittests/LookupDnDTest.h

all: cover



cover: runner.gcda
	gcovr ${GCVOR_FLAGS} --xml-pretty --output corbertura.xml
	gcovr ${GCVOR_FLAGS} -s

runner.gcda: runner
	./runner
	
runner: runner.cpp\
        GurpsDamageTable.o\
        ConvertingDndToGurps.o\
        ConfigurationsPaths.o\
        CharacterSheetSectionBase.o\
        GurpsCharacterSheetSection.o\
        GurpsArmorTable.o\
        DndCharacterSheetSection.o\
        LookupDnD.o\
        GurpsWeaponsTable.o\
        CharacterSheet.o\
        SheetLayoutLoader.o\
        SheetLayoutBlock.o\
        SheetLayoutItem.o\
        SheetLayoutItemEps.o\
        SheetLayoutItemLabel.o\
        SheetLayoutItemVariable.o\
        SheetLayoutBlockMultiColoumn.o\
        XmlSupport.o
	g++ ${CFLAGS} -o $@ $? -ltinyxml2 -lm

runner.cpp: GurpsDamageTableTest.h ConvertingDndToGurpsTest.h CharacterSheetSectionBaseTest.h GurpsCharacterSheetSectionTest.h\
   GurpsArmorTableTest.h DndCharacterSheetSectionTest.h LookupDnDTest.h GurpsWeaponsTableTest.h CharacterSheetTest.h\
   SheetLayoutLoaderTest.h
	cxxtestgen  --xunit-printer -o $@ $? 


GurpsDamageTable.o: ../GurpsDamageTable.cpp
	g++ -c ${CFLAGS} -o $@ $?

ConvertingDndToGurps.o: ../ConvertingDndToGurps.cpp
	g++ -c ${CFLAGS} -o $@ $?

ConfigurationsPaths.o: ../ConfigurationsPaths.cpp
	g++ -c ${CFLAGS} -o $@ $?

CharacterSheetSectionBase.o: ../CharacterSheetSectionBase.cpp
	g++ -c ${CFLAGS} -o $@ $?

GurpsCharacterSheetSection.o: ../GurpsCharacterSheetSection.cpp
	g++ -c ${CFLAGS} -o $@ $?

GurpsArmorTable.o: ../GurpsArmorTable.cpp
	g++ -c ${CFLAGS} -o $@ $?

GurpsWeaponsTable.o: ../GurpsWeaponsTable.cpp
	g++ -c ${CFLAGS} -o $@ $?

DndCharacterSheetSection.o: ../DndCharacterSheetSection.cpp
	g++ -c ${CFLAGS} -o $@ $?

LookupDnD.o: ../LookupDnD.cpp
	g++ -c ${CFLAGS} -o $@ $?

CharacterSheet.o: ../CharacterSheet.cpp
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutLoader.o: ../SheetLayoutLoader.cpp
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutBlockMultiColoumn.o: ../SheetLayoutBlockMultiColoumn.cpp
	g++ -c ${CFLAGS} -o $@ $?

XmlSupport.o: ../XmlSupport.cpp	
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutBlock.o: ../SheetLayoutBlock.cpp
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutItem.o: ../SheetLayoutItem.cpp
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutItemEps.o: ../SheetLayoutItemEps.cpp
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutItemLabel.o: ../SheetLayoutItemLabel.cpp
	g++ -c ${CFLAGS} -o $@ $?

SheetLayoutItemVariable.o: ../SheetLayoutItemVariable.cpp
	g++ -c ${CFLAGS} -o $@ $?
	
clean:
	-rm runner
	-rm runner.cpp
	-rm *.gcov
	-rm *.gcda
	-rm *.gcno
	-rm *.o
	-rm corbertura.xml
	-rm TEST-cxxtest.xml
	-rm core

