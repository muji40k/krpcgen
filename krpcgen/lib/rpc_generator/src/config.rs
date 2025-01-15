
pub enum Target {
    Constants,
    Types,
    Client,
    Server,
    Makefile,
    All,
}

pub struct Config<'a> {
    pub path: Option<&'a std::path::Path>,
    pub project_name: Option<String>,
    pub vla_limit: Option<usize>,
    pub target: Option<Target>,
}

impl<'a> Config<'a> {
    pub fn new() -> Self {
        Self {
            path: None,
            project_name: None,
            vla_limit: None,
            target: None,
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

pub(crate) fn project_name<'a, 'b>(cfg: &'a Option<Config<'b>>) -> &'a str {
    match cfg {
        None => "",
        Some(cfg) => match &cfg.project_name {
            None => "",
            Some(name) => name.as_str(),
        },
    }
}

pub(crate) fn vla_limit<'a>(cfg: &Option<Config<'a>>) -> usize {
    match cfg {
        None => 1024,
        Some(cfg) => match &cfg.vla_limit {
            None => 1024,
            Some(limit) => *limit,
        },
    }
}


