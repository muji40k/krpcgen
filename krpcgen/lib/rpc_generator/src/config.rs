
pub struct Config<'a> {
    pub path: Option<&'a std::path::Path>,
    pub vla_limit: Option<usize>,
}

impl<'a> Config<'a> {
    pub fn new() -> Self {
        Self {
            path: None,
            vla_limit: None,
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

