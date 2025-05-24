# NovaScript Design

The aim of this chapter is to produce an overview of the design of NovaScript.
We will **not** dive into the implementation details or choices yet.
Rather, we will immagine what we want NovaScript to look like.

> [!NOTE]
> I find this immaginative step quite useful.
> I mean, if while designing we find a particular element to be ugly or unpleasent
> to write, why bother including it in the implementation.
> Without this step, finding those elements would be more difficult.
> Even on the other end, spending time immagining the language could be a good way
> to explore possible feature and content to add to the lanauage.
> That being said, the last thing I want is for this project to blow out of reason
> with the quantity of features.
> Each one of those will need to be explored and discussed to asses the
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

Just looking at this code, it is apparent that there are a lot of consideration to do.

First of all the usage of the semicolons `;`.  
I say that using a semicolon at the end of the line feels akin to the usage of a period at the end of a sentence.  
It allows one to take a moment and reflect with the feeling of having completed one  
thing, ready to go on to the next.
Not only that, but there is also the fact that
