---
layout: default
title: Data Types
permalink: /language/types
parent: Language Reference
---

{: .no_toc }
# {{page.title}}

{: .no_toc .text-delta }
## {{site.toc_header}}

- TOC
{:toc}

## Introduction

Desmosify is not a fully statically typed language, nor does it make any guarantees about soundness, but it requires
that data types be specified in certain places (e.g. global variables, function parameters) for compile-time type
checking, as well as for program readability. These types often reflect those in Desmos, but with some compile-time
conveniences.

## Basic Types

{: #any }
### `any` — Any non-list value

{: #complex }
### `complex` — Complex number

{: .note }
This type is not yet implemented properly.

{: #real }
### `real` — Real number

{: #int }
### `int` — Integer

{: #bool }
### `bool` — Boolean value

A `true` or `false` value represented as `1` or `0`, respectively.

{: #color }
### `color` — Color value

{: #tone }
### `tone` — Tone value

{: .note }
This type is not yet implemented properly.

{: #distribution }
### `distribution` — Statistical distribution

{: .note }
This type is not yet implemented properly.

{: #str }
### `str` — Text string

{: #image }
### `image` — Image display

{: .compatibility-note }
This type is *not* available on `--target desmos-graphing3d`.

## 2D Geometric Types

{: #point }
### `(X, Y)` — 2D point

{: #polygon }
### `polygon` — Polygon object

{: .compatibility-note }
This type is *not* available on `--target desmos-graphing3d`.

{: #circle }
### `circle` — Circle object

{: .compatibility-note }
This type is *only* available on `--target desmos-geometry`.

{: #arc }
### `arc` — Circular arc object

{: .compatibility-note }
This type is *only* available on `--target desmos-geometry`.

{: .note }
This type is not yet implemented properly.

{: #line }
### `line` — Line object

{: .compatibility-note }
This type is *only* available on `--target desmos-geometry`.

{: #segment }
### `segment` — Line segment object

{: .compatibility-note }
This type is *not* available on `--target desmos-graphing`.

{: #ray }
### `ray` — Line ray object

{: .compatibility-note }
This type is *only* available on `--target desmos-geometry`.

{: #vector }
### `vector` — Vector object

{: .compatibility-note }
This type is *not* available on `--target desmos-graphing`.

## 3D Geometric Types

{: #point3d }
### `(X, Y, Z)` — 3D point

{: #triangle3d }
### `triangle3d` — 3D triangle object

{: .compatibility-note }
This type is *only* available on `--target desmos-graphing3d`.

{: .note }
This type is not yet implemented properly.

{: #sphere3d }
### `sphere3d` — 3D sphere object

{: .compatibility-note }
This type is *only* available on `--target desmos-graphing3d`.

{: #segment3d }
### `segment3d` — 3D line segment object

{: .compatibility-note }
This type is *not* available on `--target desmos-graphing`.

{: #vector3d }
### `vector3d` — 3D vector object

{: .compatibility-note }
This type is *not* available on `--target desmos-graphing`.

{: #internal_bool }
### `internal_bool` — Desmos-internal boolean value

{: .note }
This type is basically useless. It can be stored and passed around, but it cannot be used in a variable update or much
of anything else and its value cannot be displayed by the Desmos interface.

Representation of a pure boolean value in Desmos, which is only obtainable through very hacky means. It is described by
Desmos's interface as a "true/false value." This is in contrast to the [`bool`](#bool) type, which is represented as a
number.

{: #enum }
## Enumeration Types

## List Types

{: #list }
### `[T]` — List of `T`

{: #broadcastable }
### `T+` — Broadcastable `T`
