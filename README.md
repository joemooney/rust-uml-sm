# Rust UML2 StateMachine

A UML2 StateMachine implemenation in Rust programming language.

## Goals

- full support for UML 2.x specification

## Building the associated Book 

Along with documentation as part of the source code, there is a markdown book in this repo.
This book is generated using the handy mdbook crate, and published on github.io.


- Build/View the book locally: ```mdbook build --open```
- Rebuild/View loop: ```mdbook watch --open```

## Documentation


- View the book:  [https://joemooney.github.io/rust-uml-sm/](https://joemooney.github.io/rust-uml-sm/)
- Source code: [https://github.com/joemooney/rust-uml-sm/](https://github.com/joemooney/rust-uml-sm/)
-- View the book locally: [file:///home/jpm/rust/rust-uml-sm/book/index.html](file:///home/jpm/rust/rust-uml-sm/book/index.html)
- Crate documentation: ```cargo doc --open``` For a cloned git repo, this will open the documentation for the source code for the crate and its dependencies.
-- To generate documentation with private items (functions, fields, etc.)  use ```cargo doc --open --document-private-items```
