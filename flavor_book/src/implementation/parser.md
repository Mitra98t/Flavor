# Parser

In this chapter, we describe the creation of the parser to produce the ASTs from
the token list.

## Setup

Let us discuss the possible implementation choices for a parser.
There are multiple routes we could take; the standard strategy involves the
usa of some library for the _parser creation_ ([yacc](https://it.wikipedia.org/wiki/Yacc) is a good example).
To stay true to the project philosophy, we will implement everything from
scratch.  
Let us imagine the solution to this problem.

_We are working at a level of abstraction at this stage._  
The most basic behavior that the parser must incorporate is the capability to
check tokens to assess the correctness of the order in which they appear.
We can logically conclude that we will use a method like `expect_token(expected: token)`.
Assuming the presence of such method, we can then imagine how we can use it.
To check the tokens involved in the `let` definition, for instance, we can chain
multiple calls of the `expect_token` method.
This logic will be inserted in its method, let's call it `parse_let()`.
This is where the problem arises.  
The variables value, for the `let` definition, can be an expression which is
composed of multiple tokens itself.
This means that we will probably need a dedicated method to parse expressions
(`parse_expression()`).
But then again, the expression is a recursive definition (we can have the
expression `3+3+3+3+...`, meaning that to parse the operand of an operation we
can encounter the operation itself as a child: **recursion**).  
That is basically it, we just need to specify the precedence in which we want to
evaluate the operations, and then recursively call the parsing functions
accordingly.

> [!IMPORTANT]
> This kind of parser is called [Recursive Descent Parser](https://en.wikipedia.org/wiki/Recursive_descent_parser)

An important specification to do is related to the imperative structure of
Flavor.  
In the Flavor code we can define a sequence of instructions (statements separated by the
semicolon).
This structure is reflected on the implementation of the parser, which will produce
a vector of ASTs (one per statement).

## Development

**Disclamer**  
Implementing a parser is challenging because it involves understanding and
handling many elements and concepts at the same time. When building the parser
from scratch, we will add each part step by step until the system is complete.  
In this chapter, we will present the implementation and explanations of the key
aspects, often including comments that guide you through the process as if we
were building the parser together.

As we have already done for the [lexer](./lexer.md), we will start by defining
the necessary types to then use in the parser (the code is again found in
[types](https://github.com/Mitra98t/Flavor/blob/main/src/types.rs)).

More precisely, we are going to define the AST nodes first.

```rust,no_run,noplayground:types.rs
// ... Above
pub enum ASTNode {
    Body {
        nodes: Vec<ASTNode>,
    },
    If {
        guard: Box<ASTNode>,
        then_body: Box<ASTNode>,
        else_body: Option<Box<ASTNode>>,
    },
    While {
        guard: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    LetDeclaration {
        identifier: String,
        var_type: Option<Type>,
        expr: Box<ASTNode>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<(String, Type)>,
        return_type: Type,
        body: Box<ASTNode>,
    },
    Return(Box<ASTNode>),
    Break,
    FunctionCall {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    UnitLiteral,
    NumberLiteral(String),
    StringLiteral(String),
    BoolLiteral(String),
    Identifier(String),
    ArrayAccess {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
    },
    BinaryExpression {
        left: Box<ASTNode>,
        operator: String,
        right: Box<ASTNode>,
    },
    UnaryExpression {
        operator: String,
        operand: Box<ASTNode>,
        is_postfix: bool,
    },
    ExpressionStatement(Box<ASTNode>),
}
```

I understand that this collection of AST nodes might seem random or complicated.
This is because, as mentioned earlier, in this chapter i have listed the parser
in its final state. However, I did not implement everything all at once. This is
a limitation of the book format.

Let us gather some key insights from this enum of `ASTNodes`.
As discussed in [Language
Development](../language_development/language_development.md) we need to specify
the nodes with which to represnet the elements present in Flavor.
We will break down the `ASTNode` enum by analyzin g some examples.
The **literals** get their specific node (Number -> NumberLiteral, True, False ->
BooleanLiteral, ecc...).
The literals can be used as operand in **operations**, both binary and unary
(BinaryExpression, UnaryExpression).
Then comes the **statements**

From this definition alone the recursive nature of the parser structure is
apparent.
Notice how the expression (`expr`) in the `LetDeclaration` node is itself of
type `ASTNode`.

We can then define the parser as the following struct (code found in [parser](https://github.com/Mitra98t/Flavor/blob/main/src/parser.rs)):

```rust,no_run,noplayground:parser.rs
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
```

We just need the token list to check and the current position we are checking.

The interesting part follows.  
In the book we will report just a part of the parser highlighting the most
important steps and elements.
The entire code is found on the [GitHub of the project](https://github.com/Mitra98t/Flavor) (entirely open source, feel free to _contribute_).  
We will define the entry point for the parser as a public method together with
some helper functions.

```rust,no_run,noplayground:parser.rs
// ... Above

impl Parser {
    // Helper
    fn current_tok(&self) -> &Token {
        &self.tokens[self.pos]
    }

    // Helper
    fn consume_tok(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    // Helper
    fn expect_tok(&mut self, expected: TokenName) -> Result<Token, String> {
        let tok = self.current_tok();

        if tok.tok_name == expected {
            let tok = tok.clone();
            self.consume_tok();
            Ok(tok)
        } else {
            Err(format!(
                "Expected token {:?}, found {:?} ('{}')",
                expected, tok.tok_name, tok.lexeme
            ))
        }
    }

    // Entry Point
    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut nodes = Vec::new();
        while self.current_tok().tok_name != TokenName::Eof {
            nodes.push(self.parse_statement()?);
        }
        Ok(nodes)
    }
}
```

These helper methods allow us to get the current token, consume it
(shift the list starting point right one position), and to check the token with a given
`TokenName`.

Lastly, the `parse_program()` method will serve as an entry point for the
parser.
Notice the signature of the method which will return the vector of ASTs.

Now for the fun stuff, we will need the collection of parser functions to handle
the different elements of the grammar.  
First off, the `parse_statement()` method which is responsible for the parsing
of all the different statements we support in Flavor.

```rust,no_run,noplayground:parser.rs
fn parse_statement(&mut self) -> ParseProduction {
    match self.current_tok().tok_name {
        TokenName::Let => self.parse_let_statement(),
        // TODO: aliasing, fn declaration, return statement, ecc...
        _ => self.parse_expression_statement(),
    }
}
```

Currently we have little support yet, the `let` declaration is the first element
in the list which would be followed by all the other statements.
The default parsing will be `parse_expression_statement()` which is responsible
for the handling of a statement as `3+4;`.

> Why do we need such statements without _side effects_? What are side effects?
> What is the difference between statement, expression and expression-statement?
> These are all topics which are discussed in chapter [TODO]().

Let us now analyze the method for the parsing of let declarations
(`parse_let_statement()`).
This will serve as an example to understand how we can apply the grammar in the
parser to check the tokens.
We can describe the `let` statement as follows: `let identifier (':' identifier)? '='
<expr> ';'`.
This means that we expect, in order:

1. the `let` token
2. the `identifier` token
3. _optionally_, the `colon` followed by another identifier
4. the equal sign '=' token
5. a collection of tokens composing an expression
6. the semicolon `;` token
<!-- FIX: the type should be its own token?? -->

To achieve this pattern of checks we implement the following code:

```rust,no_run,noplayground:parser.rs
fn parse_let_statement(&mut self) -> Result<ASTNode, String> {
    self.expect_tok(TokenName::Let)?;
    let id_tok = self.expect_tok(TokenName::Identifier)?;

    let var_type: Option<String> = if self.current_tok().tok_name == TokenName::Colon {
        self.consume_tok();
        Some(self.parse_type()?)
    } else {
        None
    };

    self.expect_tok(TokenName::Assign)?;
    let expr = self.parse_expression()?;
    self.expect_tok(TokenName::Semicolon)?;

    // Return of the AST
    Ok(ASTNode::LetDeclaration {
        identifier: id_tok.lexeme,
        var_type,
        expr: Box::new(expr),
    })
}
```

Notice how the required tokens are checked in sequence.
The usage of the `?` is useful due to the return type of the parse functions.
The functions will return a `Result` type; if an error is present, the `?` symbol
allows to escalate it to the caller.

> For those of you that are reimplementing Flavor in other languages, this
> escalation system can be achieved by using `throw` and `try-catch` in Java for
> example.  
> Custom made solutions are also possible if not encouraged.
> We will experiment with a custom error handling and report system later in this
> book.

In this definition, the initialization value is required, while the declaration type
is optional.
This is shown in the implementation where the presence of the type is checked using an `if` statement,
and the `var_type` variable is of type `Option`, meaning it can be `None`.

> [!IMPORTANT]
> The `expect_tok()` method also returns a `Result`.
> That is so that the method will return the token if the check is positive and
> a structured error message if the check is negative.
> Having the method return a token allows for the caller to use it to compose
> the node of the AST.

In this example, the `id_tok` is stored with the return value of the
`expect_tok()` after checking if the token is an identifier.
The `id_tok` variable is then used to compile the `LetDeclaration` ASTNode.

The final important element in this example is the call to the `parse_expression()` method.
As we said prior, the grammar expects an expression after the `=` symbol.
The implementation will represent this with the calling of the
`parse_expression()` method.  
To make it absolutely clear, the structure of the `parse_expression()` method
will reseamble the on in `parse_let_statement()`.
Also checking the token sequence and calling other parse methods as necessary to
then compose an AST.
The AST obtained from the `parse_expression()` will be then used in the
`LetDeclaration` AST node in the example.

### Operator Precedence

We have talked about _precedence_ in an informal way.
In the context of **operations**, the precedence is a value to represent the
ordering in which to execute said operations.

We represent this ordering with the following function.

```rust,no_run,noplayground:parser.rs
// ... Above
fn get_precedence(token: &Token) -> Option<u8> {
    match token.tok_name {
        TokenName::Plus | TokenName::Minus => Some(10),
        TokenName::Times | TokenName::Div | TokenName::Percent => Some(20),
        TokenName::Eq | TokenName::NotEq => Some(5),
        _ => None,
    }
}
```

With this system we can associate to each operation an **priority value** and we
can then parse those operations accordingly.

The function responsible for the parsing is the following.

```rust,no_run,noplayground:parser.rs
fn parse_binary_expression(&mut self, min_prec: u8) -> ParseProduction {
    let mut left = self.parse_postfix_expression()?;

    while let Some(prec) = Self::get_precedence(self.current_tok()) {
        if prec < min_prec {
            break;
        }

        let op_tok = self.current_tok().clone();
        self.consume_tok();

        let right = self.parse_binary_expression(prec + 1)?;
        left = ASTNode::BinaryExpression {
            left: Box::new(left),
            operator: op_tok.lexeme,
            right: Box::new(right),
        }
    }
    Ok(left)
}
```

The important detail in this function are the usage of the recursion calling the
`parse_binary_expression()` method itself for the right operand of the
operation; and the usage of a precedence check to correctly stop the evaluation
of the operations.
The recursive call of the method will increase the precedence value to ensure
that the right child of an operation node is always of an higher precedence.

We will leave out all the parsing methods.
Just one more will be included in the book.
Specifically, the parsing method responsible for the parsing of terminal nodes.

```rust,no_run,noplayground:parser.rs
fn parse_primary(&mut self) -> ParseProduction {
    let tok = self.current_tok().clone();
    match tok.tok_name {
        TokenName::Number => {
            self.consume_tok();
            Ok(ASTNode::NumberLiteral(tok.lexeme))
        }
        TokenName::Identifier => {
            self.consume_tok();
            Ok(ASTNode::Identifier(tok.lexeme))
        }
        TokenName::LPar => {
            self.consume_tok();
            let expr = self.parse_expression()?;
            self.expect_tok(TokenName::RPar)?;
            Ok(expr)
        }
        _ => Err(format!("Unexpected token in expression: {:?}", tok)),
    }
}
```

Here the parsing is simpler; the token gets checked.
If it is a terminal token, then the corresponding AST node is returned.
If, instead, it is a parenthesis, then we recursively call the
`parse_expression()` method restarting the cycle.
