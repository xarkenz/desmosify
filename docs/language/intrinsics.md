---
layout: default
title: Intrinsics
permalink: /language/intrinsics
parent: Language Reference
---

{: .no_toc }
# {{page.title}}

{: .no_toc .text-delta }
## {{site.toc_header}}

- TOC
{:toc}

## Introduction

An *intrinsic* in Desmosify is a value or function starting with the "at" symbol (`@`). Intrinsics allow the programmer
to perform a variety of operations which may not be possible to represent otherwise. Most have a Desmos equivalent—for
example, `@sin` corresponds to the **sin** function.

{: .note }
Many of the planned intrinsic functions are not yet implemented, but will be coming soon. As such, there are numerous
gaps in this page.

### About the syntax used on this page

The syntax used in this reference is not necessarily valid Desmosify code, but serves to give an idea of how these
intrinsics can be used. There are a few notations which may be unfamiliar:

- `..args: type` indicates that the intrinsics is variadic—that is, it accepts an unlimited number of arguments. All of
  the arguments that correspond to `args` must be of type `type`.
- `// T: type1 | type2` indicates that `T` is a generic type and can be replaced with either `type1` or `type2`.
- `// T: any` indicates that `T` is a generic type and can be replaced with any type.
- `{description}` indicates a type described in plain English.

## Trigonometry

{: #pi }
### `@pi` — Mathematical constant *π*

```
@pi: real

// @pi => 3.14…
```

{: #tau }
### `@tau` — Mathematical constant *τ*

```
@tau: real

// @tau => 6.28…
```

{: #sin }
### `@sin` — Sine

```
@sin(theta: real+): real+

// @sin(@pi / 2) => 1.0
```

Compute the sine of `theta` radians.

{: #cos }
### `@cos` — Cosine

```
@cos(theta: real+): real+

// @cos(@pi / 2) => 0.0
```

Compute the cosine of `theta` radians.

{: #tan }
### `@tan` — Tangent

```
@tan(theta: real+): real+

// @tan(@pi) => 0.0
```

Compute the tangent of `theta` radians.

## Statistics

{: #mean }
### `@mean` — Mean/Average value

```
// T: real | (real, real) | (real, real, real)
@mean(..values: T+): T+
@mean(values: [T]): T

// @mean(3, 1, 5, 2) => 2.75
// @mean([3, 1, 5, 2]) => 2.75
```

Compute the arithmetic mean/average value in `values`.

{: #min }
### `@min` — Minimum value

```
// T: real | int | bool
@min(..values: T+): T+
@min(values: [T]): T

// @min(3, 1, 4, 2) => 1
// @min([3, 1, 4, 2]) => 1
```

Compute the minimum value in `values`. `bool` values are interpreted as 0 or 1.

{: #max }
### `@max` — Maximum value

```
// T: real | int | bool
@max(..values: T+): T+
@max(values: [T]): T

// @max(3, 1, 4, 2) => 4
// @max([3, 1, 4, 2]) => 4
```

Compute the maximum value in `values`. `bool` values are interpreted as 0 or 1.

{: #count }
### `@count` — Count values

```
@count(..values: any+): int+
@count(values: [any]): int

// @count(1, 2, 3) => 3
// @count([1 ..= 10]) => 10
```

Compute the number of values in `values`.

{: #total }
### `@total` — Total of values

```
// T: real | int | bool
// U (derived from T): real | int | int
@total(..values: T+): U+
@total(values: [T]): U

// @total(1, 2, 3) => 6
// @total([1 ..= 5]) => 15
```

Compute the sum of all values in `values`. `bool` values are interpreted as 0 or 1.

## List Operations

{: #join }
### `@join` — Join into a single list

```
// T: any
@join(..components: T+): [T]

// @join([1, 2], [3], 4) => [1, 2, 3, 4]
```

Join two or more lists/values into a single list by concatenating them in order.

{: #sort }
### `@sort` — Sort a list in ascending order

```
// K: real | int | bool
// T: any
@sort(list: [K]): [K]
@sort(list: [T], keys: [K]): [T]

// @sort([3, 1, 4, 2]) => [1, 2, 3, 4]
// -@sort(-[3, 1, 4, 2]) => [4, 3, 2, 1]
// @sort([a, b, c], [3, 1, 2]) => [b, c, a]
// -@sort([a, b, c], -[3, 1, 2]) => [a, c, b]
```

Sort `list` in ascending (increasing) order. If only `list` is provided, its values are used as the keys for sorting.
If `keys` is provided, `list` will be sorted according to the ordering of `keys`. (`keys` and `list` must have the same
number of values.) The sort used is a *stable* sort—that is, items that use the same key will remain in the same order
as they were in the original list.

To sort in descending order, the idioms `-@sort(-list)` or `-@sort(list, -keys)` can be used.

{: #shuffle }
### `@shuffle` — Randomly shuffle a list

```
// T: any
@shuffle(list: [T]): [T]
@shuffle(list: [T], seed: real): [T]

// @shuffle([1 ..= 4]) => e.g. [2, 3, 1, 4]
```

Shuffle the items in `list` using global randomness (and `seed` if it is provided).

{: #unique }
### `@unique` — Remove duplicate items in a list

```
// T: any
@unique(list: [T]): [T]

// @unique([1, 4, 1, 3, 3, 2, 3, 4]) => [1, 4, 3, 2]
```

Retain only the first of each unique item in `list`. Note that this also works for items that are not normally
comparable with `==`, such as points and colors.

## Visualizations

## Distributions

## Statistical Tests

## Calculus

## Geometry

{: #segment }
### `@segment` — Construct a segment

```
@segment(point_1: (real, real), point_2: (real, real)): segment
```

{: .compatibility-note }
This intrinsic is only available on `--target desmos-geometry`.

Construct a line segment from `point_1` to `point_2`.

{: #polygon }
### `@polygon` — Construct a polygon

```
@polygon(..points: (real, real)+): polygon+
@polygon(points: [(real, real)]): polygon
```

Construct a closed polygon using `points`.

## Properties & Measurements

## Transformations

{: #rotate }
### `@rotate` — Rotate an object about a point

```
// T: (real, real) | polygon | segment | circle | arc | line | ray | vector
@rotate(object: T+, point: (real, real)+, angle: real+): T+
```

{: .compatibility-note }
This intrinsic is only supported on `--target desmos-geometry`.

Rotate `object` about `point` by `angle` radians.

## Color

{: #rgb }
### `@rgb` — Create a color in sRGB space

```
@rgb(red: real+, green: real+, blue: real+): color+
```

{: #hsv }
### `@hsv` — Create a color in HSV space

```
@hsv(hue: real+, saturation: real+, value: real+): color+
```

`hue` is in degrees, *mod* 360. `saturation` and `value` range from 0 to 1.

{: #okhsv }
### `@okhsv` — Create a color in OkHSV space

```
@okhsv(hue: real+, saturation: real+, value: real+): color+
```

`hue` is in degrees, *mod* 360. `saturation` and `value` range from 0 to 1.

{: #oklab }
### `@oklab` — Create a color in OkLab space

```
@oklab(lightness: real+, a: real+, b: real+): color+
```

{: #oklch }
### `@oklch` — Create a color in OkLCh space

```
@oklch(lightness: real+, chroma: real+, hue: real+): color+
```

## Sound

## Number Theory

{: #e }
### `@e` — Euler's constant *e*

```
@e: real

// @e => 2.71…
```

## Complex

## Advanced

{: #width_pixels }
### `@width_pixels` — Viewport width

```
@width_pixels: real
```

{: .compatibility-note }
This intrinsic is unsupported on `--target desmos-graphing3d`.

The horizontal width of the current viewport, in logical pixels.

{: #height_pixels }
### `@height_pixels` — Viewport height

```
@height_pixels: real
```

{: .compatibility-note }
This intrinsic is unsupported on `--target desmos-graphing3d`.

The vertical height of the current viewport, in logical pixels.

{: #dt }
### `@dt` — Delta time between ticks

```
@dt: real
```

{: .note }
This intrinsic is only defined within `ticker` declarations.

The approximate time elapsed, in milliseconds, since the previous ticker tick. This time is not guaranteed to—and often
will not—match the ticker's interval, if set.

{: .warning }
If the ticker is paused for a duration of time using a `disable` action, the first tick after the ticker resumes will
include the pause time in `@dt`. This can cause strange behavior depending on what `@dt` is used for.

{: #index }
### `@index` — Clicked object index

```
@index: int
```

{: .note }
This intrinsic is only defined within `click` actions in `display` blocks.

The index of the object which was clicked by the user. If the object is in a list of objects being displayed, this is
the index of that object in the list. Otherwise, `@index` is 1.

## Desmosify

{: #enum_values }
### `@enum_values` — List all valid values of an `enum` type

```
// T: {some enum type}
@enum_values(enum_type: {type T}): [T]

// enum E { A, B, C }
// @enum_values(E) => [E.A, E.B, E.C]
```

{: #enum_value }
### `@enum_value` — Get the corresponding `enum` value

```
// T: {some enum type}
@enum_value(enum_type: {type T}, ordinal: int+): T+

// enum E { A, B, C }
// @enum_value(E, 1) => E.B
```

{: .warning }
The `ordinal` provided is currently not checked for validity, though it may be in the future. Handling invalid ordinals
must be done manually if desired.

{: #include_text }
### `@include_text` — Include a separate file as text

```
@include_text(path: str): str
```

Read the contents of a text file at `path` and convert it to a `str` at compile time. The file is expected to be encoded
in UTF-8, so the compiler may throw an error if a different encoding is used.

The `path` given is interpreted relative to the path of the current source file, *not* the current working directory
from which the compiler was invoked.

{: #include_data }
### `@include_data` — Include a separate file as a data URL

```
@include_data(path: str): str
@include_data(path: str, media_type: str): str
```

Read the contents of a binary file at `path` and convert it to a data URL at compile time. A `media_type` can be
provided to override the MIME type of the file, as encoded in the data URL (e.g. `image/png`); if not provided, the MIME
type is guessed from the file extension (this behavior is sufficient for most purposes).

The `path` given is interpreted relative to the path of the current source file, *not* the current working directory
from which the compiler was invoked.

{: #image }
### `@image` — Create an image object

```
@image(
    url: str,
    name: str,
    center: (real, real),
    width: real,
    height: real,
    opacity: real = 1.0,
    angle: real = 0.0,
    background: bool = false,
): image
```

Create an image using the provided data `url` (often obtained from [`@include_data`](#include_data)). When used in a
`display` block, this displays an image centered at `center` with the provided `width`/`height` (in units), `opacity`,
and `angle` (in radians). If `background` is set to `true`, the image is layered in the background of the graph;
otherwise, it is layered according to its order in the `display` block.

{: #concat }
### `@concat` — Concatenate strings

```
@concat(..strings: str): str

// @concat() => ""
// @concat("abc") => "abc"
// @concat("Hello", " ", "world!") => "Hello world!"
```

{: #target }
### `@target` — Compilation target name

```
@target: str

// @target => e.g. "desmos-graphing"
```

The name of the compilation target; that is, the string given for `--target` on the command line.

{: #target_symbol }
### `@target_symbol` — Convert identifier to compilation target symbol

```
@target_symbol(global: {global reference}): str
@target_symbol(action: {action reference}): str

// let my_global: i32 = 0;
// @target_symbol(my_global) => e.g. "G_{MyGlobal}"
// action my_action() {}
// @target_symbol(action my_action) => e.g. "A_{MyAction}"
```

Given a global or action identifier, get the name of the corresponding symbol on the compilation target as a string.
This can be used to, for example, refer the user to a global or action exposed in a `public` block, or for substituting
the value of a symbol in a label.

For Desmos targets, the symbol name is the identifier converted to its LaTeX form (as shown in the above examples).
