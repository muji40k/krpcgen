
pub mod config;
mod handle;
mod file;
mod types;
mod program {
    pub mod client;
    pub mod server;
}
mod make;
mod misc;

use handle::{ Handle, Type };
use file::{File, Printable, IteratorPrinter};

pub fn generate(
    definitions: impl Iterator<Item=rpc::Definition>,
    cfg: Option<config::Config<impl AsRef<std::path::Path>>>
) -> std::io::Result<()> {
    let handle = Handle::from_iter(definitions);

    [
        generate_constants,
        generate_types,
        generate_servers,
        generate_clients,
        generate_make,
    ].into_iter().try_for_each(|stage| stage(&cfg, &handle))
}

fn generate_constants(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut file = file::HFile::new(config::path(cfg).join("constants.h"))
        .expect("Filename provided");

    types::misc_constants(&mut file, types::Constants::new(cfg));
    "".chain(IteratorPrinter::from(
        handle.module.constants.iter()
    )).chain("").chain(IteratorPrinter::from(
        handle.module.programs.iter().map(|(v, progr)|
            (&progr.name, v)
        )
    )).chain("").chain(IteratorPrinter::from(
        handle.module.types.enums.iter()
    )).print(&mut file);

    file.finish();
    file.result()
}

fn generate_types(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg).join("types.h"))
        .expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/module.h>",
        "",
        "#include \"constants.h\"",
        "",
    ]).switch(types::misc_types).chain("").switch(|file| handle.order.types.iter().for_each(|tp| {
        match tp {
            Type::Typedef(name) => (
                name, handle.module.types.typedefs.get(name).expect("Was added")
            ).print(file),
            Type::Struct(name) => (
                name, handle.module.types.structs.get(name).expect("Was added")
            ).print(file),
            Type::Union(name) => (
                name, handle.module.types.unions.get(name).expect("Was added")
            ).print(file),
        }
        "".print(file);
    })).print(&mut hfile);

    hfile.finish();
    hfile.result()
}

fn generate_servers(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    generate_server_program_common(cfg, handle).and_then(|_| {
        handle.module.programs.values().try_for_each(|program| [
            generate_server_program_constants,
            generate_server_program_authentication,
            generate_server_program_module,
            generate_server_program_versions,
        ].iter().try_for_each(|f| f(cfg, handle, program)))
    })
}

fn generate_server_program_common(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("servers")
        .join("common.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/sunrpc/svc.h>",
        "",
    ]).switch(|file|
        program::server::misc::generate_thread_function_definition(handle, file)
    ).switch(|file|
        program::server::misc::generate_dispatch_definition(handle, file)
    ).print(&mut hfile);


    let mut cfile = file::CFile::new(config::path(cfg)
        .join("servers")
        .join("common.c")
    );

    IteratorPrinter::from([
        "#include <linux/kthread.h>",
        "#include <linux/freezer.h>",
        "",
        "#include <linux/sunrpc/svcsock.h>",
        "",
        "#include \"common.h\"",
        "",
    ]).switch(|file|
        program::server::misc::generate_thread_function_declaration(handle, file)
    ).chain("").switch(|file|
        program::server::misc::generate_dispatch_declaration(handle, file)
    ).print(&mut cfile);

    hfile.finish();
    hfile.result().and_then(|_| {
        cfile.finish();
        cfile.result()
    })
}

fn generate_server_program_constants(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("servers")
        .join(program.name.as_str())
        .join("constants.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include \"../../constants.h\"",
        ""
    ]).switch(|file|
        program::server::generate_program_constants(handle, file, program)
    ).print(&mut hfile);

    hfile.finish();
    hfile.result()
}

fn generate_server_program_authentication(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("servers")
        .join(program.name.as_str())
        .join("authentication.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/sunrpc/svcauth.h>",
        "",
    ]).switch(|file|
        program::server::generate_program_auth_handle_size(
            handle, file, program
        )
    ).chain("").switch(|file|
        program::server::generate_program_auth_definition(
            handle, file, program
        )
    ).print(&mut hfile);

    let mut cfile = file::CFile::new(config::path(cfg)
        .join("servers")
        .join(program.name.as_str())
        .join("authentication.c")
    );

    IteratorPrinter::from([
        "#include \"authentication.h\"",
        ""
    ]).switch(|file|
        program::server::generate_program_auth_declaraion(
            handle, file, program
        )
    ).print(&mut cfile);

    hfile.finish();
    hfile.result().and_then(|_| {
        cfile.finish();
        cfile.result()
    })
}

fn generate_server_program_module(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    let mut cfile = file::CFile::new(config::path(cfg)
        .join("servers")
        .join(program.name.as_str())
        .join("program.c")
    );

    IteratorPrinter::from([
        "#include <linux/module.h>",
        "#include <linux/kernel.h>",
        "",
        "#include <linux/sunrpc/svc.h>",
        "#include <linux/sunrpc/svc_xprt.h>",
        "",
        "#include \"../common.h\"",
        "#include \"constants.h\"",
        "#include \"authentication.h\"",
        "",
    ]).chain(IteratorPrinter::from(program.versions.values().map(|ver|
        format!("#include \"{}/version.h\"", ver.name)
    ))).chain(IteratorPrinter::from([
        "",
        "MODULE_LICENSE(\"GPL\");",
        ""
    ])).switch(|file|
        program::server::misc::generate_program_parameters(handle, file)
    ).chain("").switch(|file|
        program::server::generate_program_version_array(handle, file, program)
    ).chain("").switch(|file|
        program::server::generate_program_declaraion(handle, file, program)
    ).chain("").switch(|file|
        program::server::generate_program_entrypoint(handle, file, program)
    ).print(&mut cfile);

    cfile.finish();
    cfile.result()
}

fn generate_server_program_versions(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    program.versions.values().try_for_each(|ver| [
        generate_server_version_constants,
        generate_server_version_definition,
        generate_server_version_procedures,
    ].iter().try_for_each(|f| f(cfg, handle, ver, &program.name)))
}

fn generate_server_version_constants(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    ver: &rpc::Version,
    prog: &str,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("servers")
        .join(prog)
        .join(ver.name.as_str())
        .join("constants.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include \"../constants.h\"",
        ""
    ]).switch(|file|
        program::server::version::generate_version_constants(handle, file, ver)
    ).print(&mut hfile);

    hfile.finish();
    hfile.result()
}

fn generate_server_version_definition(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    ver: &rpc::Version,
    prog: &str,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("servers")
        .join(prog)
        .join(ver.name.as_str())
        .join("version.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/sunrpc/svc.h>",
        "",
    ]).switch(|file|
        program::server::version::generate_version_definition(handle, file, ver)
    ).print(&mut hfile);

    let mut cfile = file::CFile::new(config::path(cfg)
        .join("servers")
        .join(prog)
        .join(ver.name.as_str())
        .join("version.c")
    );

    IteratorPrinter::from([
        "#include \"version.h\"",
        "#include \"../../../types.h\"",
        "#include \"../../common.h\"",
        "#include \"../authentication.h\"",
        "#include \"constants.h\"",
        "",
        "#include \"procedures.h\"",
        "",
    ]).switch(|file|
        program::server::version::generate_version_procedures_array(handle, file, ver)
    ).chain("").switch(|file|
        program::server::version::generate_version_declaraion(handle, file, ver)
    ).print(&mut cfile);

    hfile.finish();
    hfile.result().and_then(|_| {
        cfile.finish();
        cfile.result()
    })
}

fn generate_server_version_procedures(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    ver: &rpc::Version,
    prog: &str,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("servers")
        .join(prog)
        .join(&ver.name)
        .join("procedures.h")
    ).expect("Filename provided");

    let mut xdr_cfile = file::CFile::new(config::path(cfg)
        .join("servers")
        .join(prog)
        .join(&ver.name)
        .join("procedure_xdr.c")
    );

    let mut handler_cfile = file::CFile::new(config::path(cfg)
        .join("servers")
        .join(prog)
        .join(&ver.name)
        .join("procedure_handlers.c")
    );

    IteratorPrinter::from([
        "#include <linux/sunrpc/svc.h>",
        "#include <linux/sunrpc/xdr.h>",
        "",
        "#include \"../../../types.h\"",
        "",
    ]).print(&mut hfile);

    [&mut xdr_cfile, &mut handler_cfile].into_iter().for_each(|f| {
        IteratorPrinter::from([
            "#include \"procedures.h\"",
            "",
        ]).print(f)
    });

    ver.procedures.values().for_each(|proc| {
        let need_release = program::server::procedure::procedure_need_release(handle, proc);

        types::generate_argument_wrap(handle, &mut hfile, proc).then_some(
            ""
        ).switch(|file| {
            program::server::procedure::generate_procedure_handler_definition(handle, file, proc, &ver.name);
            program::server::procedure::generate_procedure_arguments_decode_definition(handle, file, proc, &ver.name);
            program::server::procedure::generate_procedure_result_encode_definition(handle, file, proc, &ver.name);
            if need_release {
                program::server::procedure::generate_procedure_release_definition(handle, file, proc, &ver.name);
            }
        }).chain("").print(&mut hfile);

        program::server::procedure::generate_procedure_handler_declaration(handle, &mut handler_cfile, proc, &ver.name);
        "".print(&mut handler_cfile);

        program::server::procedure::generate_procedure_arguments_decode_declaration(handle, &mut xdr_cfile, proc, &ver.name);
        "".print(&mut xdr_cfile);
        program::server::procedure::generate_procedure_result_encode_declaration(handle, &mut xdr_cfile, proc, &ver.name);
        "".print(&mut xdr_cfile);
        if need_release {
            program::server::procedure::generate_procedure_release_declaration(handle, &mut xdr_cfile, proc, &ver.name);
            "".print(&mut xdr_cfile);
        }
    });

    {
        hfile.finish();
        hfile.result()
    }.and_then(|_| {
        xdr_cfile.finish();
        xdr_cfile.result()
    }).and_then(|_| {
        handler_cfile.finish();
        handler_cfile.result()
    })
}

fn generate_clients(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    generate_client_common(cfg, handle).and_then(|_| {
        handle.module.programs.values().try_for_each(|program| [
            generate_client_program_constants,
            generate_client_program_authentication,
            generate_client_program_module,
            generate_client_program_versions,
        ].iter().try_for_each(|f| f(cfg, handle, program)))
    })
}

fn generate_client_common(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join("client.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/sunrpc/clnt.h>",
        "",
    ]).switch(|file|
        program::client::misc::generate_client_misc_definition(handle, file)
    ).print(&mut hfile);


    let mut cfile = file::CFile::new(config::path(cfg)
        .join("clients")
        .join("client.c")
    );

    IteratorPrinter::from([
        "#include \"client.h\"",
        "",
    ]).switch(|file|
        program::client::misc::generate_client_misc_declaration(handle, file)
    ).print(&mut cfile);

    hfile.finish();
    hfile.result().and_then(|_| {
        cfile.finish();
        cfile.result()
    })
}

fn generate_client_program_constants(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join(program.name.as_str())
        .join("constants.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include \"../../constants.h\"",
        ""
    ]).switch(|file|
        program::client::generate_program_constants(handle, file, program)
    ).print(&mut hfile);

    hfile.finish();
    hfile.result()
}

fn generate_client_program_authentication(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join(program.name.as_str())
        .join("authentication.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/sunrpc/clnt.h>",
        "#include <linux/sunrpc/xdr.h>",
        "",
    ]).switch(|file|
        program::client::generate_program_auth_handle_size(
            handle, file, program
        )
    ).chain("").switch(|file|
        program::client::generate_program_auth_encode_definition(
            handle, file, program
        )
    ).print(&mut hfile);

    let mut cfile = file::CFile::new(config::path(cfg)
        .join("clients")
        .join(program.name.as_str())
        .join("authentication.c")
    );

    IteratorPrinter::from([
        "#include \"authentication.h\"",
        ""
    ]).switch(|file|
        program::client::generate_program_auth_encode_declaration(
            handle, file, program
        )
    ).print(&mut cfile);

    hfile.finish();
    hfile.result().and_then(|_| {
        cfile.finish();
        cfile.result()
    })
}

fn generate_client_program_module(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    let mut cfile = file::CFile::new(config::path(cfg)
        .join("clients")
        .join(program.name.as_str())
        .join("program.c")
    );

    IteratorPrinter::from([
        "#include <linux/module.h>",
        "#include <linux/kernel.h>",
        "",
        "#include <linux/sunrpc/clnt.h>",
        "",
        "#include \"../client.h\"",
        "#include \"constants.h\"",
        "",
    ]).chain(IteratorPrinter::from(program.versions.values().map(|ver|
        format!("#include \"{}/version.h\"", ver.name)
    ))).chain(IteratorPrinter::from([
        "",
        "MODULE_LICENSE(\"GPL\");",
        ""
    ])).switch(|file|
        program::client::generate_program_version_array(handle, file, program)
    ).chain("").switch(|file|
        program::client::generate_program_declaraion(handle, file, program)
    ).chain("").switch(|file|
        program::client::generate_program_entrypoint(handle, file, program)
    ).print(&mut cfile);

    cfile.finish();
    cfile.result()
}

fn generate_client_program_versions(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    program: &rpc::Program,
) -> std::io::Result<()> {
    program.versions.values().try_for_each(|ver| [
        generate_client_version_constants,
        generate_client_version_definition,
        generate_client_version_procedures,
    ].iter().try_for_each(|f| f(cfg, handle, ver, &program.name)))
}

fn generate_client_version_constants(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    ver: &rpc::Version,
    prog: &str,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(ver.name.as_str())
        .join("constants.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include \"../constants.h\"",
        ""
    ]).switch(|file|
        program::client::version::generate_version_constants(handle, file, ver)
    ).print(&mut hfile);

    hfile.finish();
    hfile.result()
}

fn generate_client_version_definition(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    ver: &rpc::Version,
    prog: &str,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(ver.name.as_str())
        .join("version.h")
    ).expect("Filename provided");

    IteratorPrinter::from([
        "#include <linux/sunrpc/clnt.h>",
        "",
    ]).switch(|file| {
        program::client::version::generate_version_procedures_array_definition(handle, file, ver);
        program::client::version::generate_version_definition(handle, file, ver);
    }).print(&mut hfile);

    let mut cfile = file::CFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(ver.name.as_str())
        .join("version.c")
    );

    IteratorPrinter::from([
        "#include \"version.h\"",
        "#include \"constants.h\"",
        "#include \"../authentication.h\"",
        "",
        "#include \"procedures.h\"",
        "",
    ]).switch(|file|
        program::client::version::generate_version_procedures_array_declaration(handle, file, ver)
    ).chain("").switch(|file|
        program::client::version::generate_version_declaraion(handle, file, ver)
    ).print(&mut cfile);

    hfile.finish();
    hfile.result().and_then(|_| {
        cfile.finish();
        cfile.result()
    })
}

fn generate_client_version_procedures(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
    ver: &rpc::Version,
    prog: &str,
) -> std::io::Result<()> {
    let mut hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(&ver.name)
        .join("procedures.h")
    ).expect("Filename provided");

    let mut xdr_cfile = file::CFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(&ver.name)
        .join("procedure_xdr.c")
    );

    let mut api_hfile = file::HFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(&ver.name)
        .join("procedure_api.h")
    ).expect("Filename provided");

    let mut api_cfile = file::CFile::new(config::path(cfg)
        .join("clients")
        .join(prog)
        .join(&ver.name)
        .join("procedure_api.c")
    );

    IteratorPrinter::from([
        "#include <linux/sunrpc/clnt.h>",
        "#include <linux/sunrpc/xdr.h>",
        "",
    ]).print(&mut hfile);

    IteratorPrinter::from([
        "#include <linux/sunrpc/clnt.h>",
        "#include <linux/sunrpc/xdr.h>",
        "",
        "#include \"../../../types.h\"",
        "",
    ]).print(&mut api_hfile);

    IteratorPrinter::from([
        "#include \"procedure_api.h\"",
        "#include \"procedures.h\"",
        "#include \"version.h\"",
        "#include \"constants.h\"",
        "#include \"../../client.h\"",
        "",
    ]).print(&mut api_cfile);

    IteratorPrinter::from([
        "#include \"procedures.h\"",
        "#include \"../authentication.h\"",
        "#include \"../../../types.h\"",
        "",
    ]).print(&mut xdr_cfile);

    ver.procedures.values().for_each(|proc| {
        types::generate_argument_wrap(handle, &mut hfile, proc).then_some(
            ""
        ).switch(|file| {
            program::client::procedure::generate_procedure_arguments_encode_definition(handle, file, proc, &ver.name);
            program::client::procedure::generate_procedure_result_decode_definition(handle, file, proc, &ver.name);
        }).chain("").print(&mut hfile);

        program::client::procedure::generate_procedure_api_definition(handle, &mut api_hfile, proc, &ver.name, prog);
        program::client::procedure::generate_procedure_release_definition(handle, &mut api_hfile, proc, &ver.name, prog);
        "".print(&mut api_hfile);

        program::client::procedure::generate_procedure_api_declaration(handle, &mut api_cfile, proc, &ver.name, prog);
        "".print(&mut api_cfile);
        program::client::procedure::generate_procedure_release_declaration(handle, &mut api_cfile, proc, &ver.name, prog)
            .then_some("").print(&mut api_cfile);

        program::client::procedure::generate_procedure_arguments_encode_declaration(handle, &mut xdr_cfile, proc, &ver.name, prog);
        "".print(&mut xdr_cfile);
        program::client::procedure::generate_procedure_result_decode_declaration(handle, &mut xdr_cfile, proc, &ver.name);
        "".print(&mut xdr_cfile);
    });

    {
        hfile.finish();
        hfile.result()
    }.and_then(|_| {
        xdr_cfile.finish();
        xdr_cfile.result()
    }).and_then(|_| {
        api_hfile.finish();
        api_hfile.result()
    }).and_then(|_| {
        api_cfile.finish();
        api_cfile.result()
    })
}

fn generate_make(
    cfg: &Option<config::Config<impl AsRef<std::path::Path>>>,
    handle: &Handle,
) -> std::io::Result<()> {
    let mut makefile = file::PlainFile::new(config::path(cfg)
        .join("Makefile")
    );

    make::generate_make(handle, &mut makefile);

    makefile.finish();
    makefile.result()
}

