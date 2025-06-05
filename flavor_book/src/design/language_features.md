# Language Features and Style

In designing a programming language, seemingly small choices—such as whether to
use semicolons or parentheses in certain contexts—can have significant
implications on the language’s readability, expressiveness, and user experience.
These choices shape not only how code looks but also how it feels to write and
maintain it.

In this chapter, we analyze the implications of such decisions and explore their
impact on the language’s style and usability. We will begin by examining the use
of semicolons and parentheses, then expand to other stylistic and syntactic
choices that influence the overall flavor of the language.

Next, we will examine design choices related to implementation aspects, such as
memory management, scopes, and more.

> [!NOTE]
> If you have a different perspective or think I missed something in any of the
> following points, please create an
> [issue](https://github.com/Mitra98t/Flavor/issues) or contribute directly on
> [GitHub](https://github.com/Mitra98t/Flavor).

## Semicolons: To Use or Not to Use?

The semicolon (`;`) is a classic statement terminator in many languages such as
C, Java, and Rust. Its presence can convey important semantic information and
affect how code is parsed and understood.

### Implications of Using Semicolons

- **Encourages Deliberate Writing:** Like a period in a sentence, semicolons
  encourage the writer to consider the end of an action or intent.
- **Clear Statement Boundaries:** Semicolons explicitly mark the end of a
  statement, helping both the compiler and the programmer to distinguish one
  instruction from another.
- **Facilitates Multiple Statements Per Line:** Semicolons allow multiple
  statements on a single line, which can be convenient but may also reduce
  readability if overused.
- **Parsing Simplicity:** From an implementation standpoint, semicolons can
  simplify parsing by clearly delimiting statements.

### Implications of Omitting Semicolons

- **Cleaner Syntax:** Languages like Python and Lua omit semicolons by default,
  resulting in cleaner, less cluttered code.
- **Reliance on Line Breaks:** The parser must rely on line breaks or
  indentation to determine statement boundaries, which can sometimes lead to
  ambiguity or subtle bugs.
- **Potential for Ambiguity:** Without explicit terminators, certain constructs
  might become harder to parse or require additional rules.

### Choosing a Semicolon Policy for Flavor

For Flavor, using semicolons as statement terminators strikes a balance between
clarity and expressiveness. It signals the end of an intention clearly while
allowing flexibility in formatting. However, we may consider omitting semicolons
in certain contexts (like the last statement in a block) to reduce verbosity—a
feature common in modern languages.

## Parentheses: When Are They Necessary?

Parentheses are traditionally used to enclose function parameters, but their
usage can vary widely across languages.

### Functions and Parentheses

- **Clarity of Invocation:** Parentheses clearly distinguish function calls from
  variable references or other expressions.
- **Parameter Grouping:** They group parameters and help the parser understand
  argument boundaries.
- **Optional Parentheses:** Some languages (e.g., Lua) allow omitting
  parentheses for function calls with a single string or table argument, making
  code more concise.

### Statements Without Parentheses

Consider the `print` statement in languages like Lua, where parentheses are
optional or omitted entirely. This can make simple statements feel more natural
and less cluttered.
Also, ensuring the absence of parentheses help differentiate function calls from
statements.

```lua:parentheses.flv
// function call
fibonacci(10);

// statement
print "Hello World!";
```

This approach allows a statement to accept only a single expression as an
argument. However, a problem arises when the statement requires multiple
arguments{{footnote:Calling them arguments is improper, but the term provides a
good intuitive understanding of the concept.}}.

For example, consider a dummy statement `foo` that needs two arguments. We can
pass a tuple to keep the _single expression as arugment for statements_ rule,
resulting in the syntax `foo (arg1, arg2)`. Note that this is not a function
call, but a statement using a tuple as its parameter.

The syntax does not clearly distinguish between these two cases in such
scenarios.  
Should we keep this syntax as it is? Or, similar to Rust's approach, should we
adopt an explicit notation to clearly differentiate these elements, like Rust
does with macros using an exclamation mark (`print!(...)`)?

### Implications for Flavor

In Flavor, we propose requiring parentheses for function calls to maintain
clarity and consistency, especially as functions become more complex. However,
for certain built-in statements or commands like `print`, we may allow
parentheses to be optional or omitted, enhancing code readability and fluency.

> [!NOTE]
> Currently, parentheses are expected only for function calls, not for
> statements.

## Type Definitions: Style, Readability, and Power

When it comes to defining types in a programming language, the choices we make
have a big impact on how the language _feels_ to write and read. It’s not just
about correctness or functionality — it’s about aesthetics, expressiveness, and
the balance between simplicity and control.

### Aesthetics: What Should Type Definitions Look Like?

Imagine you’re writing code and need to specify the type of a variable or
function parameter. How do you want that to _look_?

- Should it be verbose and explicit, like `int`, `float`, or `string` — simple,
  familiar words that almost feel like natural language?
- Or should it be more precise and low-level, like `i32`, `f64`, or `usize`,
  which convey exact size and behavior but might feel more intimidating or
  technical?
- Maybe a mix of both, where you can use simple names for everyday cases but
  also dive into detailed type specifications when you want?

For example, consider these two snippets:

```rust,no_run,noplayground:Typing Example
let x: number = 10;
let y: int = 10;
let z: i32 = 10;
```

Both declare an integer variable, but the first feels more approachable and
human-friendly, whereas the second gives you exact control over the integer’s
size and representation.
We could also use more general names like `number`, but this would require
automatically converting between different types of numbers (such as floats,
integers, and doubles) when performing operations.

### Naming: Number vs Int, Float vs f32, f64, etc.

The choice of type names influences readability and clarity:

- **Simple names** like `number`, `int`, `float` are easy to understand and
  remember — great for beginners and quick coding.
- **Detailed names** like `i32`, `u64`, `f64` provide precision, which is
  crucial when performance, memory layout, or interoperability matter.

Some languages (like Python or JavaScript) lean heavily on simple, abstract type
names, hiding low-level details. Others (like Rust or C) expose those details
upfront.

### Learning from Other Languages

- **Rust** uses explicit low-level types like `i32` and `f64`, which can be
  daunting for beginners but powerful for systems programming.
- **Go** uses simple names like `int` and `float64`, balancing ease of use with
  precision.
- **TypeScript** offers a rich type system with friendly names and advanced
  features, making it approachable yet expressive.
- **Python** largely hides type details but has recently introduced optional
  typing with simple syntax to improve readability.

### Function Types

Functions are a core building block in any programming language, and how we define their types directly affects both readability and expressiveness. In Flavor, we want function type definitions to be clear, approachable, and consistent with the overall style of the language.

A typical function definition in Flavor looks like this:

```typescript:flavor
fn foo(a: int, b: int) -> int {
  return a + b;
}
```

With this notation, the user is able to statically describe the return type of
the function as well as the parameters type.
This will allow us to specify static checks and to provide rich error messages
to the user.

### Wrapping Up

Type definitions are more than just syntax — they set the tone for how
approachable or powerful the language feels.

In the next sections, we’ll explore how these type definitions tie into the
language’s semantics and implementation, and how they influence error handling,
inference, and more.

---

# Implementation Choices: Memory Management, Scoping, and Error Handling

Now that we have explored the language’s syntax and design style, it’s time to
consider some of the fundamental decisions that affect how Flavor will actually
_work_ under the hood. Since Flavor is intended to be an interpreted language
(in its first incarnation at least), these choices shape the interpreter’s
behavior and the programmer’s experience.

This section does not dive into code specifics but focuses on the concepts,
trade-offs, and implications of different implementation strategies. The goal is
to give you a clear understanding of the possibilities so you can confidently
explore or even implement any of them yourself.

## Memory Management

How a language manages memory is one of the most important design and
implementation decisions. It affects performance, safety, ease of use, and
complexity.

> [!NOTE]
> Explicit memory management policies may not be necessary in interpreted
> languages. However, since Flavor is designed as a learning platform rather than
> a practical programming language, we will include these policies in some form.

### Manual Memory Management

- The programmer is responsible for allocating and freeing memory explicitly.
- Offers fine-grained control and potentially very efficient memory use.
- However, it is error-prone: mistakes can cause memory leaks, dangling
  pointers, or crashes.
- Languages like C use this model.

### Garbage Collection (GC)

- The interpreter automatically tracks and frees memory that is no longer in
  use.
- Removes the burden of manual memory management from the programmer, reducing
  certain classes of bugs.
- GC can introduce pauses or overhead, which might affect performance or
  responsiveness.
- Languages like Java, JavaScript, and Python use GC.

There are different GC strategies:

- **[Tracing GC](https://en.wikipedia.org/wiki/Tracing_garbage_collection):** Periodically scans memory to find unreachable objects.
- **[Reference Counting](https://en.wikipedia.org/wiki/Reference_counting):** Keeps counters for references and frees objects when
  count reaches zero.

### Stack Allocation

- Some values (like local variables) can be allocated on a stack with automatic
  cleanup when scope ends.
- This is fast and simple but limited to values with clear
  [lifetimes](https://en.wikipedia.org/wiki/Object_lifetime).
- Often combined with GC or manual management for other cases.

### Implications for Flavor

Choosing **garbage collection** (GC) simplifies programming for the user and
fits well with the flexibility of an interpreted language. Among GC techniques,
reference counting is easier to implement but has difficulties handling cyclical
references.  
Manual memory management offers powerful control but significantly
increases complexity on the user hand and risk, making it less suitable for a
language aimed at learning. However it is a fairly easy system to implement in
its most basic form.  
Meanwhile, stack allocation can improve performance for
local variables but requires careful tracking of variable lifetimes to avoid
errors.

## Scoping

Scoping rules determine how and where variables and functions are visible and
accessible, shaping both language semantics and implementation complexity.

### Lexical (Static) Scoping

- The scope of variables is determined by their position in the source code.
- Most modern languages use lexical scoping.
- Enables predictable bindings and supports closures (functions capturing
  variables from their defining environment).
- Easier to reason about and implement in an interpreter.

### Dynamic Scoping

- Variable bindings are resolved by the call stack at runtime.
- Can be more flexible but harder to predict and debug.
- Rarely used in modern mainstream languages.

### Nested Scopes and Closures

- Supporting nested scopes allows functions to be defined inside other
  functions, capturing variables from outer scopes.
- Closures are powerful for abstraction and functional programming styles.
- Implementing closures requires the interpreter to maintain environments that
  outlive their original call frames.

### Implications for Flavor

1. Flavor adopts lexical (static) scoping, meaning that the visibility of
   variables is determined by their position in the source code rather than the
   call stack at runtime. This ensures that variables declared inside a block are
   only accessible within that block and any nested inner blocks.
2. This scoping model enforces clear boundaries for variable visibility,
   preventing accidental access to variables that are out of scope. For example,
   the following code produces an error because `y` is used outside its declaring
   block:

```typescript:Scoping Example
let x = 4;

if x > 2 {
  let y = 5;
  y;
}

print y; // Error: y is not defined here
```

3. By making variable lifetimes explicit and predictable, lexical scoping
   improves both the language’s design and the programmer’s experience. Errors
   caused by undefined or shadowed variables can be detected early, helping users
   write safer and more maintainable code.
4. Nested scopes and variable shadowing are naturally supported. Inner blocks
   can declare variables with the same name as outer ones, temporarily
   overriding the outer binding within that inner scope. This adds flexibility
   while maintaining clarity.

## Error Handling

How a language handles errors influences code robustness and developer
experience.

### Compile-Time vs Runtime Errors

- **Compile-time errors** catch problems before the program runs, improving safety
  but requiring more upfront checks.
- **Runtime errors** occur during execution and must be handled gracefully to avoid
  crashes.

### Exception Handling

- Languages often provide constructs like `try/catch` to handle exceptions.
- Enables separating error-handling code from normal logic but can complicate
  control flow.

### Result Types and Explicit Handling

- Some languages (like Rust) use types like `Result` to represent success or
  failure explicitly.
- Encourages handling errors explicitly but can add verbosity.

### Implications for Flavor

TODO...

## Additional Style Decisions

Beyond semicolons and parentheses, other stylistic choices also shape the
language’s character:

### 1. **Block Delimiters**

- Using braces `{}` to delimit blocks is familiar and visually clear.
- Alternatives include indentation-based blocks (like Python), which reduce
  punctuation but complicate parsing.

---

## Summary

Small syntactic and stylistic choices have outsized effects on the language’s
usability and identity. By carefully considering semicolon usage, parentheses,
block delimiters, keywords, operators, and type annotations, we hope to better
streamline the implementation process.

Each of these implementation choices—memory management, scoping, and error
handling—comes with trade-offs that affect performance, safety, ease of use, and
complexity. There is no one-size-fits-all solution, and part of the learning
journey is understanding these trade-offs deeply.

By exploring these options thoroughly, anyone interested in programming
language development can choose the path that best fits their goals, whether for
Flavor or any other language project.

In the following chapters, we will explore these decisions in more detail and
see how they influence both the language’s design and its implementation.
