# AST parsing to retains the full structure of the code

I will document here all the usage of [mago](https://github.com/carthage-software/mago) in the project.
Mago is a toolchain in rust that allows to do a lot of things on a PHP codebase.
It's written in Rust and is very fast.

In order to scan the codebase, we need to understand the library and how to use it.

## AST Parsing 

Mago provide a [command](https://github.com/carthage-software/mago/blob/main/src/commands/ast.rs) to parse and display the AST of a PHP file.

It also provide a [command](https://github.com/carthage-software/mago/blob/main/src/commands/find.rs) to find reference of a symbol in the codebase.

### Mago 'find' command

Mago uses a ThreadInterner to store strings in a way that they can be shared between threads without duplicating them.

It uses a struct called `SourceManager` that contains the source code of the files.

For each source file, it will build an AST and find all references in that file.
Building the ast returns a `Module` and a `Program`.
The part that we care is probably the Module, it contains a specific property called `CodebaseReflection`.


## Observations

It looks like the AST parsing or other stuff are done with the walker pattern.

Maybe the goal for us is to build a `CodebaseReflection` for the whole codebase, and then use it to validate all rules.
The `CodebaseReflection` contains a merge function to merge multiple `CodebaseReflection` instances together.

The `Program` struct is the nearest representation of the current `php-parser-rs` AST that we have. (Which is currently outdated)

# MVP

The goal of the first MVP is to be able to parse the AST of a PHP workspace, and test the rule E14: Correct number of arguments in function calls.
- No vendor files are parsed.
- It only parses the files in the `src` directory.
- It does not parse the files in the `tests` directory.

Our program should help us to quickly find the method used in the file, and then check if the number of arguments is correct.