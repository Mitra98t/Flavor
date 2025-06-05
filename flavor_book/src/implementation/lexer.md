# Lexer

Going back to the introduction to this chapter
([Implementation](./implementation.md)), we reviewed the inteded behavior of the
lexer, not its specific implementation for Flavor.  
Here, in these sections, we will analyze the code behind Flavor and the
reasonings that generated it.

In this chapter, we will create a lexer that uses regular expression (regex) rules.  
The idea behind this lexer is to define a set of regex patterns that represent the tokens we want to identify.  
When a regex matches a part of the code, we generate the corresponding token and add it to the token list.  
For example, we can define the regex `[0-9]+` to find numbers in the source code.  
This regex is then tested against the source code, and if it matches a string like  
`146`, the token `(NUMBER, "146")` is generated.

## Setup

The creation of a lexer starts with the definition of the allowed syntax.
In the case of Flavor, this step was done in the
[design](../design_of_the_language.md) phase.  
Using the code as we imagined it, we can define this allowed syntax.
For instance, notice the usage of the keyword `let`, the semicolon `;` and the
colon `:`.
All these elements will correspond to tokens in the lexer.
We can repeat the process to provide a complete list of tokens in an `enum`.
We will also define the struct `Token` to represent the tuple for the token (as described in [Implementation](./implementation.md)).

## Development

First things first, we decided to use Rust for the project; we will then follow
a structured approach to development: first comes the definition of the types,
than their behaviors, and lastly we use them in the main code.

The types definition for the lexer are straightforward; we need the allowed tokens and
a struct to represent the concept of a token.

> The code necessary to do so is found in [types](https://github.com/Mitra98t/Flavor/blob/main/src/types.rs).

Let us first define the `TokenNames` to check for in the lexer.

```rust,no_run,noplayground:types.rs
pub enum TokenName {
    // Keywords
    Let, Fn, Alias,

    // Symbols
    Colon, Semicolon, // ...

    // Parentheses
    LPar, RPar, // ...

    // Complex Elements
    Number, Identifier, // ...

    // Utils
    Unknown, Eof,
}
```

We can then specify the `Token` structure.

```rust,no_run,noplayground:types.rs
// ... Above

pub struct Token {
    pub tok_name: TokenName,
    pub lexeme: String,
}
```

The usage of a `struct` to define the touple for the token may seem overkill.
That is because, at this stage, it is.  
This is simply future thinking to allow for the inclusion of more complex
elements as the column, row, and length of the lexeme.
Those elements can be useful to provide nice errors to the user.

> [!NOTE]
> I've never implemented an error system for a language.
> This means that I am assuming the need for those more comlpex elements in the
> structure.  
> I could be mistaken.

Once the definition of the useful types is done, we can start implementing the
logic.  
We will create a new file called `lexer.rs` ([lexer
file](https://github.com/Mitra98t/Flavor/blob/main/src/lexer.rs)) that we will use to create the lexer
class and its workings.

To start let us define the structure for the lexer itself.

```rust,no_run,noplayground:lexer.rs
use regex::Regex;

pub struct Lexer {
    pub tokens: Vec<Token>,
    pos: usize,
    source: String,
}
```

We leave the `tokens` public to access them from outside the lexer itself.
The `source` attribute is used to store the source code to process.
Lastly the `pos` element represents the starting position of the source code;
this means that when a snippet is consumed by the lexer (see the next part to
understand how) we will advance the position (`pos`) to indicate a new starting
point for the lexer.

The workings of the lexer are as follows:

```rust,no_run,noplayground:lexer.rs
// Above

impl Lexer {
    pub fn new(source_code: &str) -> Self {
        Lexer {
            tokens: vec![],
            pos: 0,
            source: source_code.to_string(),
        }
    }

    pub fn lexe(&mut self) {
        loop {
            let tok = self.next_token();

            self.tokens.push(tok.clone());
            if tok.tok_name == TokenName::Eof {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Token {
                tok_name: TokenName::Eof,
                lexeme: "\0".to_string(),
            };
        }

        let mut tok = Token {
            tok_name: TokenName::Unknown,
            lexeme: "".to_string(),
        };

        let mut length_of_tok: usize = 0;

        let patterns = [
            (r"let", TokenName::Let),
            (r"fn", TokenName::Fn),
            (r"alias", TokenName::Alias),
            (r":", TokenName::Colon),
            (r";", TokenName::Semicolon),
            (r"==", TokenName::Eq),
            (r"\!=", TokenName::NotEq),
            (r"\=", TokenName::Assign),
            (r"\!", TokenName::Not),
            // ...
            (r"[0-9]+", TokenName::Number),
            (r"[a-zA-Z_][a-zA-Z0-9_]*", TokenName::Identifier),
            (r"[\s\S]*", TokenName::Unknown),
        ];

        for (pattern, token_name) in patterns.iter() {
            if let Some(lexeme) = self.match_start(pattern) {
                tok.tok_name = token_name.clone();
                tok.lexeme = lexeme.to_string();
                length_of_tok = lexeme.len();
                break;
            }
        }

        self.consume_n_char(length_of_tok);

        tok
    }

    fn match_start(&self, pattern: &str) -> Option<&str> {
        let re = Regex::new(pattern).unwrap();
        if let Some(mat) = re.find(self.remaining_source()) {
            if mat.start() == 0 {
                Some(mat.as_str())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        let re = Regex::new(r"^\s+").unwrap();
        while let Some(m) = re.find(self.remaining_source()) {
            if m.start() == 0 {
                self.consume_n_char(m.end());
            } else {
                break;
            }
        }
    }

    fn remaining_source(&self) -> &str {
        &self.source[self.pos..]
    }

    fn consume_n_char(&mut self, n: usize) {
        self.pos += n;
    }
}
```

Let us break this code down.  
The core of the logic is the `next_token()` method.
This method is responsible for the production of the token.
The logic is simple, the method checks the source code against known regex rules; if
one maches, then the corresponding token is produced.  
To do so, the method uses an array of touples `(regex, token_name)`; a for loop
then iterates over those touple and checks them against the source code.  
The first that matches the source is used for the production of the token, the
loop breaks and the source code start is shifted by however many characters have
been matched.

> [!IMPORTANT]
> The sequence in which we provide the regex rules is critical for the correct
> operation of the lexer.
> If we were to check the regex for the assignment (`r"\="`) before the
> regex for the equality (`r"\=="`) we would never be able to correctly produce
> the token `Eq` resulting, instead, into two `Assign` tokens.  
> I find that this can be considered as a weakness of this kind of lexers

<!-- TODO: Cite resource for the separation of concerns -->

Surrounding this core logic we find a small collection of methods used to better
follow the separation of concerns principle.
Those methods are used to:

- ignore whitespaces, tabs and other junk characters `skip_whitespace()`
- consume the used characters in the source code advancing the starting position
  `consume_n_char`
- match the source code with the regex rule `match_start`
- get the remaining source code to process `remaining_source`

Lastly, to provide a cleaner code, we create another method called `lexe()` to
serve as an entry point for the usage of the lexer.
The body of the `lexe()` method consists only of the iterative call of the
`next_token()` method untill the end of file.
The tokens obtained in this manner are collected into the `tokens` property of
the `lexer` to then be accessed.

At this stage we can put together a simple `main()` ([main file](https://github.com/Mitra98t/Flavor/blob/main/src/main.rs)) for the project to text the
correctness of the lexer

```rust,noplayground,no_run:main.rs
fn main() {
    let code = r#"let
    fn
    alias

    foo
    47

    : ; = == != ! -> => > < >= <=
    + - * / % ( ) [ ] { }

    let foo = 3;"#;

    let mut lexer = Lexer::new(code);
    lexer.lexe();

    lexer.tokens.iter().for_each(|tok| {
        println!("{:?}", tok);
    });
}
```

Running this code will provide the following output:

```bash:output of the main execution
Token { tok_name: Let, lexeme: "let" }
Token { tok_name: Fn, lexeme: "fn" }
Token { tok_name: Alias, lexeme: "alias" }
Token { tok_name: Identifier, lexeme: "foo" }
Token { tok_name: Number, lexeme: "47" }
Token { tok_name: Colon, lexeme: ":" }
Token { tok_name: Semicolon, lexeme: ";" }
Token { tok_name: Assign, lexeme: "=" }
Token { tok_name: Eq, lexeme: "==" }
Token { tok_name: NotEq, lexeme: "!=" }
Token { tok_name: Not, lexeme: "!" }
Token { tok_name: SlimArrow, lexeme: "->" }
Token { tok_name: BoldArrow, lexeme: "=>" }
Token { tok_name: Gt, lexeme: ">" }
Token { tok_name: Lt, lexeme: "<" }
Token { tok_name: Ge, lexeme: ">=" }
Token { tok_name: Le, lexeme: "<=" }
Token { tok_name: Plus, lexeme: "+" }
Token { tok_name: Minus, lexeme: "-" }
Token { tok_name: Times, lexeme: "*" }
Token { tok_name: Div, lexeme: "/" }
Token { tok_name: Percent, lexeme: "%" }
Token { tok_name: LPar, lexeme: "(" }
Token { tok_name: RPar, lexeme: ")" }
Token { tok_name: LSqu, lexeme: "[" }
Token { tok_name: RSqu, lexeme: "]" }
Token { tok_name: LBra, lexeme: "{" }
Token { tok_name: Rbra, lexeme: "}" }
Token { tok_name: Let, lexeme: "let" }
Token { tok_name: Identifier, lexeme: "foo" }
Token { tok_name: Assign, lexeme: "=" }
Token { tok_name: Number, lexeme: "3" }
Token { tok_name: Semicolon, lexeme: ";" }
Token { tok_name: Eof, lexeme: "\0" }
```

Indeed confirming the correctness of the lexing step.
