all: ftdi

ftdi: ftdi.cpp
	clang++ $^ -o $@ $(shell libftdi1-config --cflags) $(shell libftdi1-config --libs)
