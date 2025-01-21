
use std::io::Write;

pub trait File {
    fn add_line(self: &mut Self, line: String);
    fn finish(self: &mut Self);
}

pub trait Printable {
    fn print(self: Self, file: &mut dyn File);

    fn chain<P: Printable>(self: Self, other: P) -> impl Printable
    where
        Self: Sized
    {
        ChainPrinter(self, other)
    }

    fn switch<FO: FnOnce(&mut dyn File)>(self: Self, f: FO) -> impl Printable
    where
        Self: Sized
    {
        ChainPrinter(self, f)
    }
}

impl Printable for &str {
    fn print(self: Self, file: &mut dyn File) {
        file.add_line(self.to_string());
    }
}

impl Printable for String {
    fn print(self: Self, file: &mut dyn File) {
        file.add_line(self);
    }
}

pub struct FnPrinter<P: Printable, FC: FnOnce() -> P>(FC);

impl<P: Printable, FC: FnOnce() -> P> From<FC> for FnPrinter<P, FC> {
    fn from(value: FC) -> Self {
        FnPrinter(value)
    }
}

impl<P: Printable, FC: FnOnce() -> P> Printable for FnPrinter<P, FC> {
    fn print(self: Self, file: &mut dyn File) {
        (self.0)().print(file)
    }
}

impl<P: Printable> Printable for Option<P> {
    fn print(self: Self, file: &mut dyn File) {
        match self {
            Some(inner) => inner.print(file),
            None => {},
        }
    }
}

pub struct IteratorPrinter<P: Printable, I: Iterator<Item=P>>(I);

impl<P: Printable, II: IntoIterator<Item=P>> From<II> for IteratorPrinter<P, II::IntoIter> {
    fn from(value: II) -> Self {
        IteratorPrinter(value.into_iter())
    }
}

impl<P: Printable, I: Iterator<Item=P>> Printable for IteratorPrinter<P, I> {
    fn print(self: Self, file: &mut dyn File) {
        self.0.for_each(|p| p.print(file))
    }
}

pub struct ChainPrinter<P1: Printable + Sized, P2: Printable>(P1, P2);

impl<P1: Printable, P2:Printable> Printable for ChainPrinter<P1, P2> {
    fn print(self: Self, file: &mut dyn File) {
        self.0.print(file);
        self.1.print(file);
    }
}

impl<FO: FnOnce(&mut dyn File)> Printable for FO {
    fn print(self: Self, file: &mut dyn File) {
        (self)(file)
    }
}

pub struct PlainFile<P: AsRef<std::path::Path>> {
    path: P,
    buffer: Vec<String>,
    result: std::io::Result<()>,
}

impl<P: AsRef<std::path::Path>> PlainFile<P> {
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

impl<P: AsRef<std::path::Path>> File for PlainFile<P> {
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

pub type CFile<P> = PlainFile<P>;

pub struct HFile<P: AsRef<std::path::Path>> {
    base: CFile<P>,
    finished: bool,
}

impl<P: AsRef<std::path::Path>> HFile<P> {
    pub fn new(path: P) -> Option<Self> {
        let pathr = path.as_ref();
        pathr.iter()
            .filter(|item| "." != *item)
            .map(|item| item.to_str())
            .try_fold(String::new(), |acc, item| item.map(|item| {
                acc + "_" + &item.replace(".", "_").to_uppercase()
            }))
            .map(|res| res + "_")
            .and_then(|name| {
                let mut base = CFile::new(path);

                format!{"#ifndef {name}"}.print(&mut base);
                format!{"#define {name}"}.print(&mut base);
                "".print(&mut base);

                Some(Self { base, finished: false})
            })
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

