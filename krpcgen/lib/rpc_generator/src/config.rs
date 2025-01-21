
pub struct Config<'a> {
    pub path: Option<&'a std::path::Path>,
    pub vla_limit: Option<usize>,
    pub port: Option<u16>,
    pub threads: Option<usize>,
}

impl<'a> Config<'a> {
    pub fn new() -> Self {
        Self {
            path: None,
            vla_limit: None,
            port: None,
            threads: None,
        }
    }
}

pub(crate) fn path<'a, 'b>(cfg: &'a Option<Config<'b>>) -> &'b std::path::Path {
    match cfg {
        None => std::path::Path::new("."),
        Some(cfg) => match cfg.path {
            None => std::path::Path::new("."),
            Some(p) => p,
        },
    }
}

pub(crate) fn vla_limit(cfg: &Option<Config>) -> usize {
    match cfg {
        None => 1024,
        Some(cfg) => match &cfg.vla_limit {
            None => 1024,
            Some(limit) => *limit,
        },
    }
}

pub(crate) fn port(cfg: &Option<Config>) -> u16 {
    match cfg {
        None => 0,
        Some(cfg) => match &cfg.port {
            None => 0,
            Some(port) => *port,
        },
    }
}

pub(crate) fn threads(cfg: &Option<Config>) -> usize {
    match cfg {
        None => 1,
        Some(cfg) => match &cfg.threads {
            None => 1,
            Some(threads) => *threads,
        },
    }
}


