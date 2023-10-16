# Makefile

EXE = cashgb.out

SOURCES = $(wildcard *.cpp)
HEADERS = $(wildcard *.h)
HEADS = $(wildcard *.hpp)
OBJECTS = $(SOURCES:%.cpp=%.o)


CPP = g++
CFLAGS = -lX11 -lGL -lpthread -lpng -lstdc++fs -std=c++17

program: ${OBJECTS}
	-@${CPP} ${OBJECTS} -o ${EXE} ${CFLAGS}
	-@echo Compliation Complete

%.o: %.cpp ${HEADERS} ${HEADS}
	-@${CPP} ${CFLAGS} -c $<

.PHONY: clean
clean:
	-@rm -f ${EXE}
	-@rm -f ${OBJECTS}
.PHONY: run
run: program
	-@./${EXE}
.PHONY: val
val: program
	-@valgrind ./${EXE}
