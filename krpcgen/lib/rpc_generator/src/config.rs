
pub struct Config<P: AsRef<std::path::Path>> {
    pub path: Option<P>,
    pub vla_limit: Option<usize>,
}

impl<P: AsRef<std::path::Path>> Config<P> {
    pub fn new() -> Self {
        Self {
            path: None,
            vla_limit: None,
        }
    }
}

pub(crate) fn path<'a, P: AsRef<std::path::Path>>(cfg: &'a Option<Config<P>>) -> &'a std::path::Path {
    match cfg {
        None => std::path::Path::new("."),
        Some(cfg) => match &cfg.path {
            None => std::path::Path::new("."),
            Some(p) => p.as_ref(),
        },
    }
}

pub(crate) fn vla_limit<P: AsRef<std::path::Path>>(cfg: &Option<Config<P>>) -> usize {
    match cfg {
        None => 1024,
        Some(cfg) => match &cfg.vla_limit {
            None => 1024,
            Some(limit) => *limit,
        },
    }
}

