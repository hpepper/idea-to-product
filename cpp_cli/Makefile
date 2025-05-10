CXXFLAGS =	-O0 -g -Wall -fmessage-length=0  -std=c++11
# -O0 due to debugging needs 

OBJS =	itp.o\
           DatabaseInterface.o\
           ArchitectureDocument.o\
           ArchitectureDocumentLatex.o\
		   ComponentRelationLists.o\
           ViewPacket.o\
           XmlSupport.o

# -lm : Math library.
LIBS = -ltinyxml2 -lm

TARGET =	itp

$(TARGET):	$(OBJS)
	$(CXX) -o $(TARGET) $(OBJS) $(LIBS)

all:	$(TARGET) doxygen/index.html unittesting 

unittesting:
	cd unittests; make all

clean:
	- rm -f $(OBJS) $(TARGET)
	- rm core
	cd unittests; make clean

totalclean: clean
	- rm test.*
	- rm -rf doxygen/html
	- rm -rf doxygen/latex

doxygen/index.html: 
	doxygen itp.doxygen
