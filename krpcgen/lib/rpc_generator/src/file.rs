
use std::io::Write;

pub trait File {
    fn add_line(self: &mut Self, line: String);
    fn finish(self: &mut Self);
}

pub trait Printable {
    fn print<F: File>(self: Self, file: &mut F);
}

impl<P: Printable + Copy> Printable for &P {
    fn print<F: File>(self: Self, file: &mut F) {
        (*self).print(file)
    }
}

impl Printable for &str {
    fn print<F: File>(self: Self, file: &mut F) {
        file.add_line(self.to_string());
    }
}

impl Printable for String {
    fn print<F: File>(self: Self, file: &mut F) {
        file.add_line(self);
    }
}

pub struct IteratorPrinter<P: Printable, I: Iterator<Item=P>>(I);

impl<P: Printable, II: IntoIterator<Item=P>> From<II> for IteratorPrinter<P, II::IntoIter> {
    fn from(value: II) -> Self {
        IteratorPrinter(value.into_iter())
    }
}

impl<P: Printable, I: Iterator<Item=P>> Printable for IteratorPrinter<P, I> {
    fn print<F: File>(self: Self, file: &mut F) {
        self.0.for_each(|p| p.print(file))
    }
}

pub struct CFile<P: AsRef<std::path::Path>> {
    path: P,
    buffer: Vec<String>,
    result: std::io::Result<()>,
}

impl<P: AsRef<std::path::Path>> CFile<P> {
    pub fn new(path: P) -> Self {
        Self {
            path,
            buffer: Vec::new(),
            result: Ok(()),
        }
    }

    pub fn result(self: Self) -> std::io::Result<()> {
        self.result
    }
}

impl<P: AsRef<std::path::Path>> File for CFile<P> {
    fn add_line(self: &mut Self, line: String) {
        self.buffer.push(line)
    }

    fn finish(self: &mut Self) {
        self.result = match self.path.as_ref().parent() {
            None => Ok(()),
            Some(dir) => match std::fs::create_dir_all(dir) {
                Err(err) => match err.kind() {
                    std::io::ErrorKind::AlreadyExists => Ok(()),
                    _ => Err(err),
                },
                _ => Ok(()),
            },
        }.and_then(|_| std::fs::File::create(&mut self.path))
        .map(|file| std::io::BufWriter::new(file))
        .and_then(|mut writer| self.buffer.iter().try_for_each(|line| {
            write!(writer, "{line}\n")
        }));
    }
}

pub struct HFile<P: AsRef<std::path::Path>> {
    base: CFile<P>,
    finished: bool,
}

impl<P: AsRef<std::path::Path>> HFile<P> {
    pub fn new(path: P) -> Option<Self> {
        let pathr = path.as_ref();
        match pathr.file_name() {
            None => None,
            Some(filename) => match filename.to_str() {
                None => None,
                Some(filename) => {
                    let name = format!{"_{}_", filename.replace(".", "_").to_uppercase()};
                    let mut base = CFile::new(path);

                    format!{"#ifndef {name}"}.print(&mut base);
                    format!{"#define {name}"}.print(&mut base);
                    "".print(&mut base);

                    Some(Self { base, finished: false})
                }
            }
        }
    }

    pub fn result(self: Self) -> std::io::Result<()> {
        self.base.result()
    }
}

impl<P: AsRef<std::path::Path>> File for HFile<P> {
    fn add_line(self: &mut Self, line: String) {
        if !self.finished {
            self.base.add_line(line)
        }
    }

    fn finish(self: &mut Self) {
        if !self.finished {
            "".print(&mut self.base);
            format!{"#endif"}.print(&mut self.base);
            self.finished = true;
        }

        self.base.finish();
    }
}

