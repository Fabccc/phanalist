use php_parser_rs::parser::ast::arguments::ArgumentList;
use php_parser_rs::parser::ast::control_flow::IfStatementBody;
use php_parser_rs::parser::ast::identifiers::Identifier;
use php_parser_rs::parser::ast::{Expression, Statement};

use crate::file::File;
use crate::indexers::Indexers;
use crate::results::Violation;

static CODE: &str = "E0014";
static DESCRIPTION: &str = "Correct number of arguments should be passed";

pub struct Rule {}

impl crate::rules::Rule for Rule {
    fn get_code(&self) -> String {
        String::from(CODE)
    }

    fn description(&self) -> String {
        String::from(DESCRIPTION)
    }

    fn validate(&self, _file: &File, _statement: &Statement) -> Vec<Violation> {
        vec![]
    }

    fn validate_with_indexed(
        &self,
        file: &File,
        statement: &Statement,
        indexers: &Indexers,
    ) -> Vec<Violation> {
        // Minimum = required arguments count
        // Maximum = Minimum + optional arguments
        let mut violations: Vec<Violation> = vec![];

        fn find_function_calls(
            expression: &Expression,
            violations: &mut Vec<Violation>,
            file: &File,
            indexers: &Indexers,
        ) {
            match expression {
                Expression::MethodCall(call) => match call.method.as_ref() {
                    Expression::Identifier(identifier) => {
                        trigger_violation(
                            file,
                            &call.arguments,
                            &call.target,
                            &identifier,
                            violations,
                            indexers,
                        );
                    }
                    _ => {}
                },
                Expression::StaticMethodCall(call) => {
                    // todo!("Missing info on static method call method");
                    trigger_violation(
                        file,
                        &call.arguments,
                        &call.target,
                        &call.method,
                        violations,
                        indexers,
                    );
                }
                Expression::NullsafeMethodCall(call) => match call.method.as_ref() {
                    Expression::Identifier(identifier) => {
                        trigger_violation(
                            file,
                            &call.arguments,
                            &call.target,
                            &identifier,
                            violations,
                            indexers,
                        );
                    }
                    _ => {}
                },
                Expression::Closure(closure) => {
                    for statement in &closure.body.statements {
                        analyze_statement(statement, violations, file, indexers);
                    }
                }
                _ => {}
            }
        }

        fn analyze_statement(
            statement: &Statement,
            violations: &mut Vec<Violation>,
            file: &File,
            indexers: &Indexers,
        ) {
            match statement {
                Statement::Expression(expr_stmt) => {
                    find_function_calls(&expr_stmt.expression, violations, file, indexers);
                }
                Statement::If(if_statement) => {
                    find_function_calls(&if_statement.condition, violations, file, indexers);
                    match &if_statement.body {
                        IfStatementBody::Statement {
                            statement,
                            elseifs,
                            r#else,
                        } => {
                            analyze_statement(statement, violations, file, indexers);
                            for elseif in elseifs {
                                find_function_calls(&elseif.condition, violations, file, indexers);
                                analyze_statement(&elseif.statement, violations, file, indexers);
                            }
                            if let Some(if_else_statement) = r#else {
                                analyze_statement(
                                    &if_else_statement.statement,
                                    violations,
                                    file,
                                    indexers,
                                );
                            }
                        }
                        IfStatementBody::Block {
                            colon: _,
                            statements,
                            elseifs,
                            r#else,
                            endif: _,
                            ending: _,
                        } => {
                            for statement in statements {
                                analyze_statement(statement, violations, file, indexers);
                            }
                            for elseif in elseifs {
                                find_function_calls(&elseif.condition, violations, file, indexers);

                                for else_if_statement in &elseif.statements {
                                    analyze_statement(
                                        else_if_statement,
                                        violations,
                                        file,
                                        indexers,
                                    );
                                }
                            }
                            if let Some(if_else) = r#else {
                                for if_else_statement in &if_else.statements {
                                    analyze_statement(
                                        if_else_statement,
                                        violations,
                                        file,
                                        indexers,
                                    );
                                }
                            }
                        }
                    }
                }
                Statement::Return(return_statement) => {
                    if let Some(expression) = &return_statement.value {
                        find_function_calls(expression, violations, file, indexers);
                    }
                }

                _ => {}
            }
        }

        fn trigger_violation(
            file: &File,
            argument_list: &ArgumentList,
            _target: &Expression,
            method: &Identifier,
            violations: &mut Vec<Violation>,
            indexers: &Indexers,
        ) {
            let current_argument_count = argument_list.arguments.len();

            let method_signature = match indexers.method_index.find_method(method) {
                Some(m) => m,
                None => return,
            };

            let required_argument_count: usize = method_signature.required_params;
            let maximum_argument_count: usize = method_signature.all_params;

            if current_argument_count < required_argument_count
                || current_argument_count > maximum_argument_count
            {
                let expected_args_message = if required_argument_count == maximum_argument_count {
                    format!("{}", required_argument_count)
                } else {
                    format!("{} to {}", required_argument_count, maximum_argument_count)
                };
                let message = format!(
                    "Invalid amount of arguments passed to function {} (passed {} arguments, excepted {})",
                    method_signature.method_name,
                    current_argument_count,
                    expected_args_message
                );
                let span = match method {
                    Identifier::SimpleIdentifier(si) => si.span,
                    Identifier::DynamicIdentifier(di) => di.start,
                };

                violations.push(Violation {
                    rule: String::from(CODE),
                    line: file.lines.get(span.line - 1).unwrap().to_owned(),
                    suggestion: message,
                    span: span,
                });
            }
        }

        analyze_statement(statement, &mut violations, file, indexers);

        violations
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::tests::analyse_folder_for_rule;

    use super::*;

    #[test]
    fn example() {
        let violations =
            analyse_folder_for_rule("e14", "incorrect_number_of_args_required.php", CODE);
        assert!(violations.len().eq(&1));
    }
}
