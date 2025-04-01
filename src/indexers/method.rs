use std::collections::HashMap;

use php_parser_rs::parser::ast::{
    classes::{ClassMember, ClassStatement},
    functions::{FunctionParameterList, ReturnType},
    identifiers::Identifier,
    namespaces::NamespaceStatement,
    Statement,
};

use super::Indexer;

#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub class_name: String,
    pub method_name: String,
    pub required_params: usize,
    pub all_params: usize,
    pub return_type: Option<ReturnType>,
}

pub struct MethodIndex {
    // HashMap (class_key, MethodName) -> MethodSignature
    pub methods: HashMap<(String, String), MethodSignature>,
}

impl MethodIndex {
    pub fn find_method(&self, identifier: &Identifier) -> Option<&MethodSignature> {
        match identifier {
            Identifier::SimpleIdentifier(simple_identifier) => todo!(),
            Identifier::DynamicIdentifier(dynamic_identifier) => todo!(),
        }
    }
}

impl Indexer for MethodIndex {
    fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    fn process_statements(&mut self, statements: &Vec<Statement>) {
        fn index_method(
            method_index: &mut MethodIndex,
            class_name: &String,
            method_name: String,
            parameters: &FunctionParameterList,
            return_type: Option<ReturnType>,
        ) {
            method_index.methods.insert(
                (
                    class_name.clone().to_owned(),
                    method_name.clone().to_owned(),
                ),
                MethodSignature {
                    class_name: class_name.clone().to_owned(),
                    method_name: method_name,
                    required_params: parameters
                        .parameters
                        .iter()
                        .filter(|s| !s.default.is_none())
                        .count(),
                    all_params: parameters.parameters.iter().count(),
                    return_type: return_type,
                },
            );
        }

        fn find_class_in_statement(
            method_index: &mut MethodIndex,
            namespace: String,
            statements: &Vec<Statement>,
        ) {
            for statement in statements {
                match statement {
                    Statement::Class(class) => {
                        let class_name_str = class.name.value.to_string();
                        let key = class_key(&namespace, &class_name_str);
                        // let methods_signatures = index_methods(class, class.body.members);
                        for class_member in &class.body.members {
                            match class_member {
                                ClassMember::AbstractMethod(method) => {
                                    index_method(
                                        method_index,
                                        &class_name_str,
                                        method.name.value.to_string(),
                                        &method.parameters,
                                        method.return_type.clone(),
                                    );
                                }
                                ClassMember::ConcreteMethod(method) => {}
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        for statement in statements {
            match statement {
                Statement::Namespace(ns) => match ns {
                    NamespaceStatement::Unbraced(unbraced_namespace) => find_class_in_statement(
                        self,
                        unbraced_namespace.name.value.to_string(),
                        &unbraced_namespace.statements,
                    ),
                    NamespaceStatement::Braced(braced_namespace) => match &braced_namespace.name {
                        Some(namespace_identifier) => find_class_in_statement(
                            self,
                            namespace_identifier.value.to_string(),
                            &braced_namespace.body.statements,
                        ),
                        _ => {}
                    },
                },
                _ => {}
            }
        }
    }
}

fn class_key(namespace: &str, class_name: &str) -> String {
    namespace.replace("\\", "_") + class_name
}
