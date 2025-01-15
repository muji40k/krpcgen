
pub mod config;
mod handle;
mod file;
mod types;

use handle::{ Handle, Type };
use file::{File, Printable, IteratorPrinter};

pub fn testies(definitions: impl Iterator<Item=rpc::Definition>, cfg: Option<config::Config>) -> std::io::Result<()> {
    let path = config::path(&cfg);
    let handle = Handle::from_iter(definitions);

    generate_constants(&cfg, &handle)
        .and_then(|_| generate_types(&cfg, &handle))
        .and_then(|_| {
            let mut file = file::HFile::new(path.join("testies.h"))
                .expect("Filename provided");

            file::IteratorPrinter::from([
                "#include \"types.h\"",
                "",
                "void test(void) {",
                "    int rc = 0;",
                "",
            ]).print(&mut file);
            handle.module.programs.values().map(|pr|
                pr.versions.values().map(|proc| proc.procedures.values())
            ).flatten().flatten().enumerate().for_each(|(i, proc)| {
                let name = format!("v{i}");
                format!("    {};", types::asc::declaration(&name, &proc.return_type)).print(&mut file);
                types::generate_encode_statement(&handle, &mut file,
                    &proc.return_type, &name, Some("rc"), Some(4),
                );
                "".print(&mut file);
            });
            "}".print(&mut file);

            file.finish();
            file.result()
        })
}

pub fn generate(definitions: impl Iterator<Item=rpc::Definition>, cfg: Option<config::Config>) -> std::io::Result<()> {
    let handle = Handle::from_iter(definitions);

    [
        generate_constants,
        generate_types,
    ].into_iter().try_for_each(|stage| stage(&cfg, &handle))
}

fn generate_constants(
    cfg: &Option<config::Config>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut file = file::HFile::new(config::path(cfg).join("constants.h"))
        .expect("Filename provided");

    types::misc_constants(&mut file, types::Constants::new(cfg));
    "".print(&mut file);

    IteratorPrinter::from(
        handle.order.constants.iter().map(|name|
            (name, handle.module.constants.get(name).expect("Was added"))
        )
    ).print(&mut file);
    "".print(&mut file);

    IteratorPrinter::from(
        handle.module.programs.iter().map(|(v, progr)|
            (&progr.name, v)
        )
    ).print(&mut file);
    "".print(&mut file);

    IteratorPrinter::from(
        handle.order.types.iter().filter_map(|tp| match tp {
            Type::Enum(name) => Some((
                name,
                handle.module.types.enums.get(name).expect("Was added")
            )),
            _ => None,
        })
    ).print(&mut file);

    file.finish();
    file.result()
}

fn generate_types(
    cfg: &Option<config::Config>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg).join("types.h"))
        .expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/module.h>",
        "",
    ]).print(&mut hfile);
    types::misc_types(&mut hfile);
    "".print(&mut hfile);

    handle.order.types.iter().for_each(|tp| {
        match tp {
            Type::Typedef(name) => (
                name, handle.module.types.typedefs.get(name).expect("Was added")
            ).print(&mut hfile),
            Type::Struct(name) => (
                name, handle.module.types.structs.get(name).expect("Was added")
            ).print(&mut hfile),
            Type::Union(name) => (
                name, handle.module.types.unions.get(name).expect("Was added")
            ).print(&mut hfile),
            _ => return,
        }
        "".print(&mut hfile);
    });

    hfile.finish();
    hfile.result()
}

