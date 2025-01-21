
pub(crate) enum Type {
    Typedef(String),
    Struct(String),
    Union(String),
}

pub(crate) struct TypeDefinitions {
    pub typedefs: indexmap::IndexMap<String, rpc::Type>,
    pub structs: indexmap::IndexMap<String, rpc::Struct>,
    pub unions: indexmap::IndexMap<String, rpc::Union>,
    pub enums: indexmap::IndexMap<String, rpc::Enum>,
}

pub(crate) struct Module {
    pub constants: indexmap::IndexMap<String, rpc::Value>,
    pub types: TypeDefinitions,
    pub programs: indexmap::IndexMap<rpc::Value, rpc::Program>
}

pub(crate) struct DefinitionOrder {
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
                self.module.constants.insert(name, value);
            },
            rpc::Definition::Typedef(name, tp) => {
                self.order.types.push(Type::Typedef(name.clone()));
                self.module.types.typedefs.insert(name, tp);
            },
            rpc::Definition::Enum(name, en) => {
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
                constants: indexmap::IndexMap::new(),
                types: TypeDefinitions {
                    typedefs: indexmap::IndexMap::new(),
                    structs: indexmap::IndexMap::new(),
                    unions: indexmap::IndexMap::new(),
                    enums: indexmap::IndexMap::new(),
                },
                programs: indexmap::IndexMap::new(),
            },
            order: DefinitionOrder {
                types: Vec::new(),
            },
        };

        iter.into_iter().for_each(|def| handle.read_definition(def));

        handle
    }
}

