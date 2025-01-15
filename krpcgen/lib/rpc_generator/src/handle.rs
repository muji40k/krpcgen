
pub(crate) enum Type {
    Typedef(String),
    Struct(String),
    Union(String),
    Enum(String),
}

pub(crate) struct TypeDefinitions {
    pub typedefs: std::collections::HashMap<String, rpc::Type>,
    pub structs: std::collections::HashMap<String, rpc::Struct>,
    pub unions: std::collections::HashMap<String, rpc::Union>,
    pub enums: std::collections::HashMap<String, rpc::Enum>,
}

pub(crate) struct Module {
    pub constants: std::collections::HashMap<String, rpc::Value>,
    pub types: TypeDefinitions,
    pub programs: std::collections::HashMap<rpc::Value, rpc::Program>
}

pub(crate) struct DefinitionOrder {
    pub constants: Vec<String>,
    pub types: Vec<Type>,
}

pub(crate) struct Handle {
    pub module: Module,
    pub order: DefinitionOrder,
}

impl Handle {
    fn read_definition(self: &mut Self, def: rpc::Definition) {
        match def {
            rpc::Definition::Const(name, value) => {
                self.order.constants.push(name.clone());
                self.module.constants.insert(name, value);
            },
            rpc::Definition::Typedef(name, tp) => {
                self.order.types.push(Type::Typedef(name.clone()));
                self.module.types.typedefs.insert(name, tp);
            },
            rpc::Definition::Enum(name, en) => {
                self.order.types.push(Type::Enum(name.clone()));
                self.module.types.enums.insert(name, en);
            },
            rpc::Definition::Struct(name, st) => {
                self.order.types.push(Type::Struct(name.clone()));
                self.module.types.structs.insert(name, st);
            },
            rpc::Definition::Union(name, un) => {
                self.order.types.push(Type::Union(name.clone()));
                self.module.types.unions.insert(name, un);
            },
            rpc::Definition::Program(value, program) => {
                self.module.programs.insert(value, program);
            },
        }
    }
}

impl FromIterator<rpc::Definition> for Handle {
    fn from_iter<T: IntoIterator<Item = rpc::Definition>>(iter: T) -> Self {
        let mut handle = Self {
            module: Module {
                constants: std::collections::HashMap::new(),
                types: TypeDefinitions {
                    typedefs: std::collections::HashMap::new(),
                    structs: std::collections::HashMap::new(),
                    unions: std::collections::HashMap::new(),
                    enums: std::collections::HashMap::new(),
                },
                programs: std::collections::HashMap::new(),
            },
            order: DefinitionOrder {
                constants: Vec::new(),
                types: Vec::new(),
            },
        };

        iter.into_iter().for_each(|def| handle.read_definition(def));

        handle
    }
}

