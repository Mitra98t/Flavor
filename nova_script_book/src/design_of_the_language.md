# NovaScript Design

The aim of this chapter is to produce an overview of the design of NovaScript.
We will **not** dive into the implementation details or choices yet.
Rather, we will imagine what we want NovaScript to look like.

> [!NOTE]
> I find this imaginative step quite useful.
> I mean, if while designing we find a particular element to be ugly or unpleasent
> to write, why bother including it in the implementation.
> Without this step, finding those elements would be more difficult.
> Even on the other hand, spending time imagining the language could be a good way
> to explore possible feature and content to add to the lanauage.
> That being said, the last thing I want is for this project to blow out of reason
> with the quantity of features.
> Each one of those will need to be explored and discussed to assess the
> feasability and the reasons behind it.
> If these discussions produce positive expectations, then we will try and
> implement the thing.

In the following piece of code we will explore the syntax for NovaScript.

```typescript:example.nova
// variable declaration with explicit type
let x: int = 10;

// variable declaration with implicit type
let y = 20; // implicitly int

// implicit casting ??
20 + 20.0 // int + float ??

// Parenthesis or no parenthesis
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
```

Just by looking at this code, it is apparent that there are a lot of considerations to do.

First of all the usage of the semicolons `;`.  
I say that using a semicolon at the end of the line feels akin to the usage of a period at the end of a sentence.  
It incentivizes one to take a moment and reflect while feeling like they have completed one  
thing, ready to go on to the next.
Not only that, but there is also the fact that the semicolon can convey
semantical informations.
The presence of the semicolon might represent that the operation found on its
left is a **statement** (do not worry, we will get into what that means later in
this book); while its absence could indicate an **expression**.
Even so, we could argue about the need for the semicolon to differenciate two
different version of the `if`.
Looking at the following code will present the poblem.

```typescript:if_as_statement_or_expression.nova
// If as a statement
let x;
if foo > 5 {
    x = "bigger";
}
else {
    x = "smaller";
};

// If as an expression
let x = if foo > 5 {
    "bigger"
}
else {
    "smaller"
};
```
