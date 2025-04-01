use method::MethodIndex;
use php_parser_rs::parser::ast::Statement;

use crate::file::File;

pub mod method;

pub trait Indexer {
    /// Initialize the indexer
    fn new() -> Self
    where
        Self: Sized;

    /// Index a PHP file
    fn index_file(&mut self, file: &File) {
        // Get the file's declarations
        self.process_statements(&file.ast);
    }

    /// Process PHP statements to extract information
    fn process_statements(&mut self, statements: &Vec<Statement>);
}

pub struct Indexers {
    pub method_index: MethodIndex,
}

impl Indexers {

    pub fn new() -> Self {
        Self {
            method_index: MethodIndex::new()
        }
    }

    pub fn index_file(&mut self, file: &File){
        self.method_index.index_file(file);
    }
}
