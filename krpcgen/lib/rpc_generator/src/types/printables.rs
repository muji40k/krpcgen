
use super::asc;
use crate::file::{ File, Printable, IteratorPrinter };

impl Printable for (&String, &rpc::Value) {
    fn print(self: Self, file: &mut dyn File) {
        format!("#define {} {}", self.0, asc::value(&self.1)).print(file)
    }
}

impl Printable for (&String, &rpc::Type) {
    fn print(self: Self, file: &mut dyn File) {
        format!("typedef {};",
            asc::declaration(self.0, &asc::fulltype(self.1))
        ).print(file)
    }
}

impl Printable for (&String, &rpc::Struct) {
    fn print(self: Self, file: &mut dyn File) {
        IteratorPrinter::from(std::iter::once(
            format!("struct {} {{", self.0)
        ).chain(self.1.iter().map(|(field, tp)|
            format!("    {};", asc::declaration(field, &asc::fulltype(tp)))
        )).chain(std::iter::once(
            format!("}};")
        ))).print(file);
    }
}

impl Printable for (&String, &rpc::Union) {
    fn print(self: Self, file: &mut dyn File) {
        IteratorPrinter::from([
            format!("struct {} {{", self.0),
            format!("    {};", asc::switching_declaraion(&self.1.value, &self.1.switch_type)),
            format!("    union {{"),
        ].into_iter().chain(self.1.arms.values().map(|(field, tp)|
            format!("        {};", asc::declaration(field, &asc::fulltype(tp)))
        )).chain(self.1.default.as_ref().map(|(field, tp)|
            format!("        {};", asc::declaration(field, &asc::fulltype(tp)))
        )).chain([
            format!("    }} {}_u;", self.0),
            format!("}};"),
        ])).print(file);
    }
}

impl Printable for (&String, &rpc::Enum) {
    fn print(self: Self, file: &mut dyn File) {
        IteratorPrinter::from(std::iter::once(
            format!("enum {} {{", self.0)
        ).chain(self.1.iter().map(|(id, value)| value.as_ref().map(|v|
            format!("    {id} = {},", asc::value(v))
        ).unwrap_or_else(||
            format!("    {id},")
        ))).chain(std::iter::once(
            format!("}};")
        ))).print(file)
    }
}

