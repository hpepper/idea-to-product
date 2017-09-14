# See: https://www.gnu.org/software/make/manual/html_node/Automatic-Variables.html

CFLAGS = -Wall -g -I.. -std=c++11 -fprofile-arcs -ftest-coverage 
GCVOR_FLAGS = -r .. --exclude unittests/runner.cpp\
                    --exclude unittests/XmlLookupTest.h

all: cover



cover: runner.gcda
	gcovr ${GCVOR_FLAGS} --xml-pretty --output corbertura.xml
	gcovr ${GCVOR_FLAGS} -s

runner.gcda: runner
	./runner
	
runner: runner.cpp	XmlSupport.o
	g++ ${CFLAGS} -o $@ $^ -ltinyxml2 -lm

runner.cpp: XmlLookupTest.h
	cxxtestgen  --xunit-printer -o $@ $? 


XmlSupport.o: ../XmlSupport.cpp
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

