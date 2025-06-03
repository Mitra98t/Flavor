# NovaScript

<p align="center">
  <img src="./assets/simple_code.png" alt="Image 1" width="60%" style="display: inline-block; margin-right: 10px;" />
  <img src="./assets/aliasing_example.png" alt="Image 2" width="38%" style="display: inline-block;" />
</p>

_NovaScript_ is a programming language built to serve as a **learning platform** for
people interested in language development.

The structure of the project is based on the collaborative creation of a
programming language, _NovaScript_, documented in a sort of diary ([the Book](https://mitra98t.github.io/NovaScript/introduction.html)).

## Philosophy

Someone interested in language development could come and take a look at the
book to understand and follow the creation of a language.
Not only understanding how it works, but also understanding the reasoning behind the
implementation choices and hopefully learning something.
The Book serves to show the process behind the creation of a language showing
the challenges as they come up.

## Contributing

All contributions are welcome, at the moment I have to decide on guidelines to
establish.
In the meanwhile I will personally check the pull requests and try to give
appropriate feedback.

### Book Contributions

In the directory `./nova_script_book/` you will find [the Book](https://mitra98t.github.io/NovaScript/introduction.html).
To **contribute** to the book you will need to have [mdbook](https://rust-lang.github.io/mdBook/).
Mdbook is a website creator to build beautiful documentations website using
markdown file.

To successfully see the book while writing, you will need two plugins:

- [mdbook-callouts](https://crates.io/crates/mdbook-callouts) for the beautiful
  [obsidian style callouts](https://help.obsidian.md/callouts)
- [mdbook-codename](https://crates.io/crates/mdbook-codename) to display the
  file name or header in the code snippets

Once everything has been installed, cd into the book directory and run `mdbook
serve --open`

### Code Contributions

In the `./src/` directory you will find the source code for **NovaScript**.
To run the **NovaScript** engine, simply run `cargo run` in the root directory
of the project.

> You will need to have Rust on your system and Cargo to manage the project.
