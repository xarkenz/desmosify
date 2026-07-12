---
layout: default
title: Syntax
permalink: /language/syntax
parent: Language Reference
---

{: .no_toc }
# {{page.title}}

{: .no_toc .text-delta }
## {{site.toc_header}}

- TOC
{:toc}

## Top-level Declarations

{: #let-definition }
### `let` — Value/Function definition

```
let <identifier>: <type> = <value>;
let <identifier>(<identifier>: <type>, <...>): <type> = <value>;

// Examples
let my_value: int = 7;
let add(x: real, y: int): real = x + y;
```

{: #var-definition }
### `var` — Variable definition

```
var <identifier>: <type> = <value>;
var timer <identifier>: <type> = <value>;
```

{: #action-definition }
### `action` — Action definition

```
action <identifier>(<identifier>: <type>, <...>) {
    <action>,
    <...>
}
```

{: #enum-definition }
### `enum` — Enumeration type definition

```
enum <identifier> {
    <identifier>,
    <...>
}
```

{: #ticker-declaration }
### `ticker` — Ticker action declaration

```
ticker { <action>, <...> }
ticker (<interval>) { <action>, <...> }
```

{: #public-declaration }
### `public` — User-facing content declaration

```
public {
    <expression>;
    action <action-identifier>(<argument>, <...>);
    <...>
}
```

{: #display-declaration }
### `display` — Displayable content declaration

```
display {
    <value>:
        <attribute>(<argument>, <...>),
        <attribute> { <action>, <...> },
        <...>;
    <...>
}
```

## Actions

### Variable update

```
<var-identifier> := <value>
```

### Action call

```
action <action-identifier>(<argument>, <...>)
```

### Compound action

```
{ <action>, <...> }
```

### Conditional

```
if <condition> then <action>
[elif <condition> then <action>]
[...]
[else <action>]
```

### The `disable` action

```
disable
```

## Expressions

### Literals

A representation of a known value of some type. There are integer literals (e.g. `57`) for type `int`, real number
literals (e.g. `5.7`, `1e10`) for type `real`, `true` and `false` for type `bool`, and string literals (e.g. `"hello"`)
for type `str`. `infinity` is a literal representing infinity for either `real` or `int`, and `undefined` is a literal
representing an undefined value of any type.

### Identifier

A word starting with either a letter or underscore and consisting of letters, digits, or underscores (e.g. `my_name3`).
Note that keywords cannot be used as identifiers.

### Action identifier

### Intrinsic identifier

### Grouping parentheses

### Operations

### Point from components

### List from items

### List from range

### List from repeated value

### List comprehension

### List filter

### Indexing a list

### Function call

### Conditional/Piecewise

### Scoped definition

### Inline action
