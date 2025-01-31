KVER := $(shell uname -r)
KDIR := /lib/modules/$(KVER)/build

obj-m := example_server.o example_api.o usage.o

example_server-objs := \
    servers/example/authentication.o \
    servers/example/program.o \
    servers/example/initial/procedure_handlers.o \
    servers/example/initial/procedure_xdr.o \
    servers/example/initial/version.o \
    servers/common.o

example_api-objs := \
    clients/example/authentication.o \
    clients/example/program.o \
    clients/example/initial/procedure_api.o \
    clients/example/initial/procedure_xdr.o \
    clients/example/initial/version.o \
    clients/client.o

.PHONY: build clean default

default: build

build:
	make -C $(KDIR) M=$(shell pwd) modules

clean:
	make -C $(KDIR) M=$(shell pwd) clean
