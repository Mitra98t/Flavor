# Flavor Imagining the First Design

The aim of this chapter is to provide an overview of the design of Flavor. We
will **not** dive into implementation details or choices yet. Rather, we will
imagine what we want Flavor to look like.

**Disclaimer**

This chapter is not intended to present the final syntax and semantics of
Flavor. We are simply imagining the language, proposing ideas, and discussing
them.

> [!NOTE] I find this imaginative step very useful. If, while designing, we
> find a particular element ugly or unpleasant to write, why include it?
> Without this step, identifying such elements would be more difficult. On the
> other hand, spending time imagining the language is a great way to explore
> possible features and additions. That said, the last thing I want is for this
> project to become overwhelmed with too many features. Each proposed feature
> will need to be explored and discussed to assess its feasibility and
> rationale. If these discussions produce positive expectations, we will try to
> implement the feature.

## First Syntax Creation

I will leave the following text and code unchanged for the remainder of the
book's creation process. For context, I wrote this content without much thought
during the initial design phases. My intention in keeping it as is, basically
without review, is to show the _very first_ ideas that shaped the language's
aesthetics. The consequences of this initial design are discussed in the next
section.

In the following code snippet, we explore the syntax for Flavor.

```typescript:example.flv
// variable declaration with explicit type
let x: int = 10;

// variable declaration with implicit type
let y = 20; // implicitly int

// TODO: check if we want implicit casting
// implicit casting
20 + 20.0 // int + float

print x;

if x > 2 {
    // code...
} else
{
    // code...
}

fn is_even (a: int) -> bool {
    return a % 2 == 0;
}

let lambda: int -> nothing = <x> {
    print x;
};

alias function fn;

function fibonacci (x:int) -> int {
    if x == 1 || x == 2 {
        return 1;
    } else {
        return fibonacci(x-1) + fibonacci(x-2);
    };
}

let fibonacci_lambda = <x:int> -> int { // fibonacci_lambda: <int> -> int
    if x == 1 || x == 2 {
        return 1;
    } else {
        return fibonacci_lambda(x-1) + fibonacci_lambda(x-2);
    };
};
```

Just by looking at this code, it is clear that many considerations need to be
made.

## Design Choices and Reasons

When discussing the aesthetic choices behind Flavor, I will focus on
**consistency**.

> A language is **consistent** if its behavior and syntax can be understood
> without needing to be explicitly taught. For example, if we always use `( )`
> to enclose parameters, `.` to access methods and properties, and `[ ]` to
> access elements in lists or sets, then we naturally expect to access the
> second element of a tuple using `tuple[1]`, not `tuple(1)` or `tuple.1`. This
> is what I mean by **consistency** in a programming language, and all
> definitions and choices we make should follow this principle.

Moreover, the syntax should be fluent and enjoyable to write. To provide a
practical example, consider these equivalent codes from various languages:

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

All these codes perform the same task: a simple console print. As you can see,
there are both simpler and more complex ways of writing this instruction.

_Is there a best way to write the print command?_  
I cannot say that one is absolutely better than the others; however, I do have
personal preferences. I like using the `!` to indicate a macro in the Rust
example (`println!("...");`), because it immediately clarifies what is being
used (macro vs function call). But I also find the simplicity of Lua (`print
"..."`) very appealing.  
I'm confident that for **Flavor** we will avoid taking
inspiration from heavy languages like Java.

### Aliasing

An interesting idea that came up while showing the syntax to a friend is the
possibility to add aliases such as `alias function fn`, possibly even adding
multiple aliases at once like `alias [function, func] fn`. This would allow
users to customize the language themselves. I have not yet considered the
consequences and implementation challenges of such a system. We can postpone
these considerations until after we have a basic implementation.

## Design Ideas Wrap Up

This step was intentionally exploratory; the idea was just to generate ideas.
As of now, the only things I know I want to incorporate are:

- some basic typing notation
- variable declaration
- functions and lambdas notation

Hopefully, that is enough to start having fun with development. In the next
chapter, we will explore the first step of the implementation.
