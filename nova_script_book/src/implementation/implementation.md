# Implementation

To start actually implementing the language we must discuss a bit about the
elements that usually compose an engine.  
This section will focus on a generic overview; the detail of the NovaScript
implementation are discussed in the dedicated chapters (i.e. [Lexer](./lexer.md)).

A language's engine is usually devided into three sections:

1. Lexical analisys
2. Parsing
3. Interpretation or Compilation

> [!WARNING]
> These components and the way they are described here are just an introduction,
> surely not complete.
> There are multiple different ways one could go about the creation of a
> programming language.

From a high level prospective, we can think of these steps as:

1. Formalizing the content of the source code in a structure we can work with
2. Checking if the source code follows the grammar rules we defined and, if so,
   producing a custom representation of the code to encode important information
3. Executing the logic described in the source code in a virtual environment
   (interpretation) or translating the source code into lower level code such
   as assembly (compilation)

We will go deeper into these topics as they become relevant in our work, but for now
let's focus on the lexical analisys performed by the lexer.

## The Lexer

As stated, the lexer is responsible for the formalization of the source code
into a structure we can better work with programmatically.
The usual choice for this structure is an array of _tokens_.

So **what is a token?**  
We can define a token as a tuple containing an identifier and the source code
that generated it.
The code `3 + 4`, once analyzed by the lexer, will produce the following token
list:

```typescript:tokens
token_list = [
    (NUMBER, "3"),
    (BIN_OP, "+"),
    (NUMBER, "4"),
]
```

Naturally, we must have defined the tokens previously to use them as shown.
Here is where our design choices comes into play for the first time: suppose we
want to define a language that does not use the symbol `^`; that is fair and it
means that we will not define the token corresponding to that symbol.  
This concept is expandible to more complex elements.

Let elaborate on this example further so that we can reference it in a more
complete manner later in this section.
To do so we will use the more complex expression `3 + 4 * 5`.

```typescript:Lexing example
let code = "3 + 4 * 5";

let tokens = Lexer.lexe(code);

/* Tokens
(NUMBER, "3"),
(BIN_OP, "+"),
(NUMBER, "4"),
(BIN_OP, "*"),
(NUMBER, "5"),
*/
```

How the lexer does this processing to the source code is _not that relevant_.
There are multiple ways in which we can acomplish this result.  
The more common is the usage of **single character scanning system**.
The logic is that the lexer scans the source code one character at the time and
checks it to see if it correspond to a token or to the start of a token.
For example, if the character is the symbol `;`, we produce the token `SEMICOLON`.  
If the character is the letter `f`, it might be part of an identifier.  
In this case, the lexer continues scanning the following characters (looking for alphanumeric ones) and,  
once finished, produces the token `IDENTIFIER`.

## The Parser

Todo...
