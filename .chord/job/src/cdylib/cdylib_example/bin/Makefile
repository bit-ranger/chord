# Which compiler
CC = gcc
# Where to install
INSTDIR = /usr/local/bin
# Where are include files kept
INCLUDE = .
# Options for development
CFLAGS = -g -Wall -ansi -pedantic

all: example

example: cdylib_example.c
	$(CC) $(CFLAGS) -shared cdylib_example.c -o $(HOME)/.chord/lib/libcdylib_example.so


