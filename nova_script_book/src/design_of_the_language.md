# NovaScript Imagining the First Design

The aim of this chapter is to produce an overview of the design of NovaScript.
We will **not** dive into the implementation details or choices yet.
Rather, we will imagine what we want NovaScript to look like.

**Disclaimer**

This chapter is not intended to show the syntax and semantics of NovaScript.
We are just imagining the language proposing ideas and discussing them.

> [!NOTE]
> I find this imaginative step quite useful.
> I mean, if while designing we find a particular element to be ugly or unpleasant
> to write, why bother including it in the implementation.
> Without this step, finding those elements would be more difficult.
> Even on the other hand, spending time imagining the language could be a good way
> to explore possible features and content to add to the language.
> That being said, the last thing I want is for this project to get out of hand
> with the quantity of features.
> Each one of those will need to be explored and discussed to assess the
> feasibility and the reasons behind it.
> If these discussions produce positive expectations, then we will try and
> implement the thing.

## First Syntax Creation

I will leave the following text and code unchanged for the remainder of the book
creation.
For context, I wrote this stuff without much thought during the first design
phases.
My intention in keeping it as is, basically without review, is to show the first
ever ideas that went into the language aesthetics.
The consequences of this first design are discussed in the next section.

In the following piece of code we will explore the syntax for NovaScript.

```typescript:example.nova
// variable declaration with explicit type
let x: int = 10;

// variable declaration with implicit type
let y = 20; // implicitly int

// implicit casting
// TODO: check if we want implicit casting
20 + 20.0 // int + float

// Parentheses or no parentheses
print x;

if x > 2 {
    // code...
}
else {
    // code...
}

fn is_even (a: int) -> int {
    return a % 2 == 0;
}

let lambda: int -> void = <x:int> {
    print x;
};

alias function fn;

function fibonacci (x:int) -> int {
    if x == 1 || x == 2 {
        return 1;
    }
    else {
        return fibonacci(x-1) + fibonacci(x-2);
    };
}

let fibonacci_lambda = <x:int> -> int { // fibonacci_lambda: <int> -> int
    if x == 1 || x == 2 {
        return 1;
    }
    else {
        return fibonacci(x-1) + fibonacci(x-2);
    };
}

```

Just by looking at this code, it is apparent that there are a lot of considerations to make.

## Design choices and reasons

When discussing the aesthetic choices behind NovaScript I will focus on
**consistency**.

> The language is **consistent** if its behavior and syntax can be understood
> without needing to be explicitly taught.  
> For example, if we always use `( )` to enclose parameters, `.` to access
> methods and properties and `[ ]` to access elements in lists or sets, then
> we naturally expect to access the second element of a tuple using `tuple[1]`,
> not `tuple(1)` or `tuple.1`.  
> This is what I mean by **consistency** in a programming language,
> and all definitions and choices we make should follow this principle.

Moreover, the syntax should be fluent and fun to write.
To provide a practical example, let us consider a series of equivalent codes of
various languages:

```java
System.out.println("Hello World!");
```

```javascript
console.log("Hello World!");
```

```rust,no_run,noplayground:rust
println!("Hello World!");
```

```lua
print "Hello World!"
```

All these codes perform the same task: a simple console print.  
As you can see, there are both simpler and more complex ways of writing this instruction.
_Is there a best way to write the print command?_  
I certainly can't say that one is absolutely better than the others; however, I do have personal preferences.
I like using the `!` to indicate a macro in the Rust example (`println!("...");`),
because it immediately clarifies what is being used (macro vs function call).
But I also find the simplicity of Lua (`print "..."`) very appealing.
I'm confident that for **NovaScript** we will avoid taking inspiration from heavy languages like Java.

### Semicolons

First of all the usage of the semicolons `;`.  
I say that using a semicolon at the end of the line feels akin to the usage of a period at the end of a sentence.  
It incentivizes one to take a moment and reflect.
Furthermore, the usage of the semicolon represents the end of an action, the end
of an intention described through code.
Not only that, but there is also the fact that the semicolon can convey
semantic information.
The presence of the semicolon might indicate that the operation to its left is a **statement**
(do not worry, we will get into what that means later in
this book); while its absence could indicate an **expression**.

### Parentheses

To tackle the quetion of _parentheses or no parentheses_, we have to recognize
that we can use the parentheses to convey specific semantical information.  
We have a fair bit of freedom here.
For instance, we might associate the absence of the parentheses with the
_macros_ (assuming we want to include them).

Writing the code for the _Fibonacci Sequence_ function (_see [the code example](./design_of_the_language.md#first-syntax-creation)_), I had to come up with
the syntax for the typing of functions and for the definition of lambdas.  
Looking at the code, I quite enjoy the use of the less than and greater than (`< >`) to
enclose the parameters of the lambdas.
Furthermore, that syntax can be reused for the type definition of functions.
A lambda function's type, using this syntax, can be defined as `<param_type> ->
return_type`.
However, I feel like we will need more practical testing to decide the usability
evaluation of the `< >` symbols.

### Aliasing

An interesting idea that came out while showing the syntax to a friend is the
possibility to add aliases as `alias function fn`, possibly even adding multiple
aliases at once like `alias [function, func] fn`.
The result is the possibility for the customization of the language by the user
itself.  
I didn't think enough in the future to evaluate the consequences and
implementation challenges for such system.
We can postpone these elements to after we have a basic implementation.

## Design ideas wrap up

This step was intentionally messy, the idea was just to explore ideas.
As of now, the only thing that I know I want to incorporate are:

- some basic typing notation
- variable declaration
- functions and lambdas notation

Hopefully that is enough to start having fun with the development.  
In the next chapter we will explore the first step of the implementation.
