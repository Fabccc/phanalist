use php_parser_rs::parser::ast::try_block::CatchBlock;
use php_parser_rs::parser::ast::Statement;

use crate::file::File;
use crate::results::Violation;

static CODE: &str = "E0002";
static DESCRIPTION: &str = "Empty catch";
static SUGGESTION: &str = "There is an empty catch. It's not recommended to catch an Exception without doing anything with it.";

pub struct Rule {}

impl crate::rules::Rule for Rule {
    fn get_code(&self) -> String {
        String::from(CODE)
    }

    fn description(&self) -> String {
        String::from(DESCRIPTION)
    }

    fn validate(&self, file: &File, statement: &Statement) -> Vec<Violation> {
        let mut violations = Vec::new();

        if let Statement::Try(s) = statement {
            for catch in &s.catches {
                let CatchBlock { start, body, .. } = catch;
                if body.is_empty() {
                    violations.push(self.new_violation(file, SUGGESTION.to_string(), *start));
                }
            }
        };

        violations
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::tests::analyze_file_for_rule;

    use super::*;

    #[test]
    fn empty_catch() {
        let violations = analyze_file_for_rule("e2/empty_catch.php", CODE);

        assert!(violations.len().gt(&0));
        assert_eq!(
            violations.first().unwrap().suggestion,
            SUGGESTION.to_string()
        );
    }

    #[test]
    fn non_empty_catch() {
        let violations = analyze_file_for_rule("e2/non_empty_catch.php", CODE);

        assert!(violations.len().eq(&0));
    }
}
