CXXFLAGS =	-O0 -g -Wall -fmessage-length=0  -std=c++11
# -O0 due to debugging needs 

OBJS =		itp.o\
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

doxygen/index.html: 
	doxygen itp.doxygen
