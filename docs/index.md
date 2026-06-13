---
layout: home
title: Home
permalink: /
nav_order: 1
---

# **Desmosify**

Write code, compile and run in Desmos.

```
public {
    "Current fibonacci number";
    num_b;
    "Get next number";
    action next();
}

var num_a: int = 0;
var num_b: int = 1;

action next() {
    num_a := num_b,
    num_b := num_a + num_b,
}
```

![Compiler output in Desmos for above code](fibonacci.png)

Desmosify is a programming language which is based around the structure of a
[Desmos]({{site.desmos_url}}/) graph, and which can be compiled into such a graph.

[Desmodder](https://www.desmodder.com/) (unaffiliated) has a similar tool in the form of text mode; however, unlike
Desmodder's text mode, the goal of Desmosify is *not* to have 1-to-1 parity with Desmos graphs. Instead, Desmosify uses
its own defined semantics, and provides additional compile-time features such as type checking and constant evaluation
before generating Desmos graphs. It also is designed to work locally, which makes it more self-contained, though it does
make debugging harder.
