all: ftdi

ftdi: ftdi.cpp
	clang++ $^ -o $@ -g $(shell libftdi1-config --cflags) $(shell libftdi1-config --libs)
