
use crate::{
    handle,
    file::{
        File,
        Printable,
        IteratorPrinter,
    }
};

pub fn generate_make(handle: &handle::Handle, file: &mut dyn File) {
    IteratorPrinter::from([
        "KVER := $(shell uname -r)",
        "KDIR := /lib/modules/$(KVER)/build",
        "",
    ]).switch(|file|
        generate_make_mod_targets(handle, file)
    ).chain(IteratorPrinter::from([
        ".PHONY: build clean default",
        "",
        "default: build",
        "",
        "build:",
        "	make -C $(KDIR) M=$(shell pwd) modules",
        "",
        "clean:",
        "	make -C $(KDIR) M=$(shell pwd) clean",
    ])).print(file)
}

fn generate_make_mod_targets(handle: &handle::Handle, file: &mut dyn File) {
    let mut piter = handle.module.programs.values();
    let first = piter.next().expect("Non empty definition");
    let mut next = piter.next();

    match &next {
        None => format!("obj-m := {0}_server.o {0}_api.o", first.name),
        Some(_) => format!("obj-m := {0}_server.o {0}_api.o \\", first.name),
    }.print(file);

    while let Some(current) = next {
        next = piter.next();
        match &next {
            None => format!("         {0}_server.o {0}_api.o", current.name),
            Some(_) => format!("         {0}_server.o {0}_api.o \\", current.name),
        }.print(file);
    }

    "".print(file);

    IteratorPrinter::from(
        handle.module.programs.values().map(|progr| IteratorPrinter::from([
            format!("{}_server-objs := \\", progr.name),
            format!("    servers/{}/authentication.o \\", progr.name),
            format!("    servers/{}/program.o \\", progr.name),
        ]).chain(IteratorPrinter::from(
            progr.versions.values().map(|ver| IteratorPrinter::from([
                format!("    servers/{}/{}/procedure_handlers.o \\", progr.name, ver.name),
                format!("    servers/{}/{}/procedure_xdr.o \\", progr.name, ver.name),
                format!("    servers/{}/{}/version.o \\", progr.name, ver.name),
            ]))
        )).chain("    servers/common.o").chain(""))
    ).print(file);

    IteratorPrinter::from(
        handle.module.programs.values().map(|progr| IteratorPrinter::from([
            format!("{}_api-objs := \\", progr.name),
            format!("    clients/{}/authentication.o \\", progr.name),
            format!("    clients/{}/program.o \\", progr.name),
        ]).chain(IteratorPrinter::from(
            progr.versions.values().map(|ver| IteratorPrinter::from([
                format!("    clients/{}/{}/procedure_api.o \\", progr.name, ver.name),
                format!("    clients/{}/{}/procedure_xdr.o \\", progr.name, ver.name),
                format!("    clients/{}/{}/version.o \\", progr.name, ver.name),
            ]))
        )).chain("    clients/client.o").chain(""))
    ).print(file);
}


