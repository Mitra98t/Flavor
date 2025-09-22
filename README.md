# Flavor

<!-- TODO: Update images -->
<p align="center">
  <img src="./assets/simple_code.png" alt="Image 1" width="60%" style="display: inline-block; margin-right: 10px;" />
  <img src="./assets/aliasing_example.png" alt="Image 2" width="38%" style="display: inline-block;" />
</p>

_Flavor_ is a programming language built to serve as a **learning platform** for
people interested in language development.

The structure of the project is based on the collaborative creation of a
programming language, _Flavor_, documented in a sort of diary ([the Book](https://mitra98t.github.io/Flavor/introduction.html)).

## Philosophy

Someone interested in language development could come and take a look at the
book to understand and follow the creation of a language.
Not only understanding how it works, but also understanding the reasoning behind the
implementation choices and hopefully learning something.
The Book serves to show the process behind the creation of a language showing
the challenges as they come up.

## Try Flavor!

Simply run `cargo run` in the root of the repository to execute Flavor.

The executable will ask for a source file, you can use the provided test files
in the `./test_files/` directory as examples.
Simply run `cargo run ./test_files/mega.flv` to execute the mega test file.

You could also write your own `.flv` file and run it the same way.

## Flavor Language Quickstart

Flavor programs live in `.flv` files and execute top to bottom. Use `cargo run <path-to-file.flv>` from the repository root (or follow the prompt from `cargo run` with no arguments) to try the snippets below.

### Hello Flavor

Create a file `hello.flv` with the following code:

```flv
print "Hello from Flavor!";
```

Run it with `cargo run hello.flv` and the interpreter will print the greeting.

### A Tour of the Syntax

The example below combines declarations, loops, conditionals, arrays, and higher-order functions. Save it as `tour.flv` and run `cargo run tour.flv` to see the output.

```flv
fn scale_and_sum(values: [int], factor: int) -> int {
    let total: int = 0;
    let idx: int = 0;
    let count: int = 3;
    while idx < count {
        if values[idx] % 2 == 0 {
            total = total + values[idx];
        }
        idx++;
    }
    return total * factor;
}

fn choose(flag: bool) -> (int) -> int {
    if flag {
        return <value: int> -> int {
            return value;
        };
    } else {
        return <value: int> -> int {
            return value * -1;
        };
    }
}

let numbers: [int] = [2, 5, 8];
let scaler = choose(false);
let result = scaler(scale_and_sum(numbers, 2));
print "Result=", result, ", first element=", numbers[0];
```

This program prints `Result=-20, first element=2`: the even values are summed, doubled, and finally negated by a function returned from `choose`. The `count` variable matches the length of the array so the loop visits every element.

### Core Syntax Reference

- `let name[: type] = expression;` declares a mutable binding. Omit the type when Flavor can infer it from the right-hand side.
- Numbers are 64-bit integers; arithmetic uses `+ - * / %` and comparison operators `== != < <= > >=` return `bool` values.
  - Support for floating-point numbers has been added in the latest version
    with 64-bit floats.
  - Strings can now be concatenated with `+`
- Boolean logic uses `true`, `false`, `&&`, `||`, and `!`.
- Functions require parameter and return types: `fn name(param: type) -> return_type { ... }`. Use `return value;` to exit a function early.
- Anonymous functions are expressions: `<value: int> -> int { return value * 2; }` can be stored in variables or returned, enabling higher-order patterns.
- Arrays are typed with `[element_type]` and created with `[item1, item2]`. Index into arrays with `values[index]`, and chain indices for nested arrays.
- `while condition { ... }` repeats until the condition is `false`. Inside loops you can use `break;` to exit and the postfix operators `counter++` or `counter--` to update integers.
- `if condition { ... } else { ... }` branches on boolean expressions; the `else` block is optional.
- `print expr1, expr2, ...;` evaluates each expression, converts it to text, and writes the concatenation to standard output.

## Contribution Guidelines

Thank you for your interest in contributing to this project! To ensure a smooth
collaboration, please follow these guidelines:

### How to Contribute

1. **Fork the repository and create your branch** from main:
   `git checkout -b feature/your-feature`
2. **Make your changes** with clear commit messages.
3. **Test your changes**.
4. **Submit a Pull Request** describing your changes and why they are needed.

### Pull Request Process

- Ensure your PR targets the main branch.
- Include tests if applicable.
- Keep your PR focused on a single issue or feature.
- Be responsive to feedback and update your PR accordingly.

### Book Contributions

In the directory `./flavor_book/` you will find [the Book](https://mitra98t.github.io/Flavor/introduction.html).
To **contribute** to the book you will need to have [mdbook](https://rust-lang.github.io/mdBook/).
Mdbook is a website creator to build beautiful documentations website using
markdown file.

To successfully see the book while writing, you will need two plugins:

- [mdbook-callouts](https://crates.io/crates/mdbook-callouts) for the beautiful
  [obsidian style callouts](https://help.obsidian.md/callouts)
- [mdbook-codename](https://crates.io/crates/mdbook-codename) to display the
  file name or header in the code snippets
- [mdbook-footnote](https://github.com/daviddrysdale/mdbook-footnote) to allow
  for the usage of footnotes

Once everything has been installed, cd into the book directory and run `mdbook
serve --open` to see the book while writing.

To **build the book** just run `mdbook build` inside the `./flavor_book/`
directory.
This way you will update the `./docs/` folder which will be used in the pages.

### Code Contributions

In the `./src/` directory you will find the source code for **Flavor**.
To run the **Flavor** engine, simply run `cargo run` in the root directory
of the project.

> You will need to have [Rust](https://www.rust-lang.org/it) on your system and [Cargo](https://doc.rust-lang.org/cargo/) to manage the project.

#### Code Style

- Follow the existing code style and conventions.
  - I'm using the normal Rust notation validated by [clippy](https://github.com/rust-lang/rust-clippy)
- Include comments where necessary.
- Use the issue tracker to report bugs or request features.
- Provide detailed information and steps to reproduce issues.
