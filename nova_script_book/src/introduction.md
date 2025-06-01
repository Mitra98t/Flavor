# Introduction

Ciao

In this book, we will follow my attempt at buinding a simple programming
language.
It is not my first time attempting such a project, but it is the first time
doing it in a structured way while sharing the process and the code.
The other attempts were more of a learning experience; one time producing a
lexer and parser, another time producing a simple interpreter for a university
language course.
I am missing the whole picture, so I will try to cover all the steps this time.
Moreover, I have never built a compiler, so this will be a new experience for
me.

To specify some things before we start:

- This project is not meant to be a learning resource, but rather a learning
  platform; the idea of sharing this exeprience is guided by the hope that it
  will help others learn from my mistakes and successes while learning
  from suggestions and critics along the way myself.
- This book will primarily show my reasoning process and the decisions I make
  about language design and implementation.
  I will try to keep the explanations linear and clear (don't make me promise).
- I will try to keep the code as clean and readable as possible, but I cannot
  guarantee that it will be perfect.
- I will be using Rust for reasons that we will discuss.
- The implementation will be done from scratch -- as much as possible --
  limiting the use of libraries to the bare minimum.
- Calling it a book sounds pretentious; I will use this terminology purely because
  I'm using [mdbook](https://rust-lang.github.io/mdBook/index.html) to write it
  and book appears to be the name used.

Lastly, I **appreciate** any feedback, suggestion, or contribution to the project.
I will try to keep the code on GitHub up to date with the content of the book, so you can
follow along with the code as we progress through the chapters.

Without further ado, let's get started!

## Who am I?

I am a computer science student with a passion for programming languages and who
cannot focus on one thing for too long.
I have been learning about prgramming languages creation for a while (mainly for
my degree) and I wanted to put my knowledge into practice.

Please note that English is not my first language, so there might be some mistakes
in the text.
I might confuse the usage of 'I' and 'we' to describe the subject behind the process, and I am sure
that the sequence of tenses will not be perfect.
I hope that at some point the usage of the word 'I' will be wrong due to the
contributions of others, but for now, I will use it to refer to myself (at least
in the Introduction).

I find criticism very helpful and I like environments where I can learn
from others -- and hopefully where others can learn a thing or two from me --
so I will be happy to receive any feedback or suggestions.

## Who are you?

If you found this book, there is the good chance that you are interested in
language development and design.
I do not want to be so presumptuous as to assume that you will learn from this
resource even if you are a seasoned developer, but I hope that you will find
something useful in it.

The target audience I have in mind is someone who has some programming
exeprience, but is not necessarily an expert and, more importantly, who is
quite new to the topic of programming languages.

## How to read this book

The book is structured as a sort of _diary_ journaling the development of NovaScript.
For that reason the chapter will not follow a logical structure, but instead
will follow the production timeline.
This means that we can go back to describing the design of the language at any moment
during development to accurately follow my steps.
The implementation on the other hand will be structured based on the code
itself; the chapter related to the **[Lexer](./implementation/lexer.md)**, for instance, will describe the lexer's
implementation mimiking the _wiki_ of a project.  
To reiterate, the [implementation](./implementation/implementation.md) section will be structured and organized
logically, while the rest of the book will follow the development timeline.

If you are following the production of this book, then you might find the
content of this paragraph rather unsatisfying.
I will expand on this paragraph when I need to include a new notation in the
book.

**Look out for the call-outs!**
I will insert call-outs in the book, each one means something.

> [!NOTE]
> Here I will inlcude notes, mainly my reasonings and thoughts

> [!QUESTION]
> Here I will note stuff to be studied more or that i do not fully understand
> yet

> [!IMPORTANT]
> Here I will describe important sections to look for (usually highlighting
> possible mistakes)

> Add more of those here...

**Syntax highlighting!**
To highlight the syntax of NovaScript throughout the examples and the snippets
in this book, we will use other languages' highlighting (at least until i figure
out how to create custom highlighting for NovaScript) so it might be imperfect.

## Some design choices

The first, and most predominant, choice is the language we will be using for
development.
I have chosen Rust for a few reasons.
First, I am quite enjoying the language, I find its consistency and safety features
very appealing; the strong typing system promotes good design at an early stage.
There could be a counter argument to this, mainly saying that Rust puts so much
focus on types that, after spending a lot of time resolving errors arising from
types inconsistencies, you will have forgotten what the program was supposed to
do, but it surely runs without any errors.
I do not think that this is a problem, and even so, there are multiple languages
one could use for this project so feel free to experiment.
Second, Rust offers the possibility to write low-level code that, in the remote
case that I manage to finish this project implementing a compiler, could be
really useful.
And lastly, Rust is a language that is gaining popularity and has a growing
community, so I feel like it is a good choice given the shared nature of this
project.

The second choice is about the decision to implement the language from scratch.
I want to understand the process of language design and implementation; the goal
is not to create a production-ready, feature-rich and performant language; the
goal is, instead, to face the challenges someone before me would have faced, and
to come up with solutions that I am reasonably happy with.

## NovaScript

**NovaScript**, the language protagonist of this book.  
The name is not meant to represent anything; I just find it sci-fi enough so
that is seems useful.

NovaScript will be a interpreted lanugage (unless i manange to write a compiler)
offering a strict typing system, type inference, function and lambdas, and --
possibly -- classes and objects.
I will admit that the features NovaScript will offer are to be decided and will  
greatly depend on my implementation.
The few listed prior are just what I _know_ that I want to tackle in this
journey.

The aesthetic of NovaScript will be designed with the concern of consistency
in mind -- going back to [the decision to use
Rust in Some design choices](./introduction.md#some-design-choices).
I have tried a not-so-small collection of languages, and I have found a series of  
elements that I like and don't like in the syntaxes I have learned.
NovaScript's syntax will be based on the intersection of those elements.
