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

- `..args: type` indicates that the intrinsic is variadic—that is, it accepts an unlimited number of arguments. All of
  the arguments that correspond to `args` must be of type `type`.
- `T: type1 | type2` indicates that `T` is a generic type and can be replaced with either `type1` or `type2`.
- `T: any` indicates that `T` is a generic type and can be replaced with any type.
- `{description}` indicates a type described in plain English.

## Trigonometry

{: #pi }
### `@pi` — Mathematical constant π

```
@pi: real

// @pi => 3.14…
```

{: #tau }
### `@tau` — Mathematical constant τ

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

{: #cos }
### `@cos` — Cosine

```
@cos(theta: real+): real+

// @cos(@pi / 2) => 0.0
```

{: #tan }
### `@tan` — Tangent

```
@tan(theta: real+): real+

// @tan(@pi) => 0.0
```

{: #csc }
### `@csc` — Cosecant

```
@csc(theta: real+): real+
```

{: #sec }
### `@sec` — Secant

```
@sec(theta: real+): real+
```

{: #cot }
### `@cot` — Cotangent

```
@cot(theta: real+): real+
```

{: #arcsin }
### `@arcsin` — Inverse sine

```
@arcsin(x: real+): real+
```

{: #arccos }
### `@arcsin` — Inverse cosine

```
@arccos(x: real+): real+
```

{: #arctan }
### `@arctan` — Inverse tangent

```
@arctan(x: real+): real+
```

{: #arctan2 }
### `@arctan2` — Inverse tangent with two arguments

```
@arctan2(y: real+, x: real+): real+

// @arctan2(0, 1) => 0.0
// @arctan2(0, -1) => @pi
// @arctan2(-5, 0) => -@pi/2
// @arctan2(0, 0) => 0.0
```

Compute the angle from the origin to `(x, y)` in the range (-π, π].

{: #arccsc }
### `@arccsc` — Inverse cosecant

```
@arccsc(x: real+): real+
```

{: #arcsec }
### `@arcsec` — Inverse secant

```
@arcsec(x: real+): real+
```

{: #arccot }
### `@arccot` — Inverse cotangent

```
@arccot(x: real+): real+
```

{: #sinh }
### `@sinh` — Hyperbolic sine

```
@sinh(theta: real+): real+
```

{: #cosh }
### `@cosh` — Hyperbolic cosine

```
@cosh(theta: real+): real+
```

{: #tanh }
### `@tanh` — Hyperbolic tangent

```
@tanh(theta: real+): real+
```

{: #csch }
### `@csch` — Hyperbolic cosecant

```
@csch(theta: real+): real+
```

{: #sech }
### `@sech` — Hyperbolic secant

```
@sech(theta: real+): real+
```

{: #coth }
### `@coth` — Hyperbolic cotangent

```
@coth(theta: real+): real+
```

## Calculus

{: #e }
### `@e` — Euler's constant *e*

```
@e: real

// @e => 2.71…
```

{: #exp }
### `@exp` — Exponent of *e*

```
@exp(x: real+): real+

// @exp(2) => @e ** 2.0
```

{: #ln }
### `@ln` — Natural logarithm

```
@ln(x: real+): real+

// @ln(@e) => 1.0
```

{: #log }
### `@log` — Logarithm

```
@log(base: real+, x: real+): real+

// @log(@e, @e) => @ln(@e) => 1.0
```

## Number Theory

{: #lcm }
### `@lcm` — Least common multiple

```
@lcm(..values: int+): int+
@lcm(values: [int]): int

// @lcm(4, 7) => 28
// @lcm(4, 6) => 12
// @lcm([1 ..= 5]) => 60
// @lcm([]) => undefined
```

{: #gcd }
### `@gcd` — Greatest common denominator

```
@gcd(..values: int+): int+
@gcd(values: [int]): int

// @gcd(4, 7) => 1
// @gcd(4, 6) => 2
// @gcd([6, 9, 24]) => 3
// @gcd([]) => undefined
```

{: #ceil }
### `@ceil` — Round toward positive infinity

```
@ceil(x: real+): int+

// @ceil(1.0) => 1
// @ceil([1.1, 1.5, 1.7]) => [2, 2, 2]
// @ceil(-1.5) => -1
```

{: #floor }
### `@floor` — Round toward negative infinity

```
@floor(x: real+): int+

// @floor(1.0) => 1
// @floor([1.1, 1.5, 1.7]) => [1, 1, 1]
// @floor(-1.5) => -2
```

{: #round }
### `@round` — Round to nearest integer

```
@round(x: real+): int+

// @round(1.0) => 1
// @round([1.1, 1.5, 1.7]) => [1, 2, 2]
// @round(-1.5) => -1
```

{: #round_digits }
### `@round_digits` — Round to nearest decimal place

```
@round_digits(x: real+, decimal_digits: int+): real+

// @round_digits(@pi, 2) => 3.14
// @round_digits(@pi, 0) => 3.0
// @round_digits(5678.9, -2) => 5700.0
```

{: #abs }
### `@abs` — Absolute value

```
T: real | int
@abs(x: T+): T+

// @abs(5) => 5
// @abs(-5) => 5
// @abs(0) => 0
```


{: #sign }
### `@sign` — Sign value

```
@sign(x: real+): int+

// @sign(12.3) => 1
// @sign(-12.3) => -1
// @sign(0.0) => 0
```

{: #sqrt }
### `@sqrt` — Square root

```
@sqrt(x: real+): real+

// @sqrt(4) => 2.0
```

{: #cbrt }
### `@cbrt` — Cube root

```
@cbrt(x: real+): real+

// @cbrt(8) => 2.0
```

{: #nth_root }
### `@nth_root` — Nth root

```
@nth_root(x: real+, n: real+): real+

// @nth_root(16, 4) => 2.0
```

## Complex

## List Operations

{: #join }
### `@join` — Join into a single list

```
T: any
@join(..components: T+): [T]

// @join([1, 2], [3], 4) => [1, 2, 3, 4]
```

Join two or more lists/values into a single list by concatenating them in order.

{: #sort }
### `@sort` — Sort a list in ascending order

```
K: real | int | bool
T: any
@sort(list: [K]): [K]
@sort(list: [T], keys: [K]): [T]

// @sort([3, 1, 4, 2]) => [1, 2, 3, 4]
// -@sort(-[3, 1, 4, 2]) => [4, 3, 2, 1]
// @sort([a, b, c], [3, 1, 2]) => [b, c, a]
// @sort([a, b, c], -[3, 1, 2]) => [a, c, b]
```

Sort `list` in ascending (increasing) order. If only `list` is provided, its values are used as the keys for sorting.
If `keys` is provided, `list` will be sorted according to the ordering of `keys`. (`keys` and `list` must have the same
number of values.) The sort used is a *stable* sort—that is, items that use the same key will remain in the same order
as they were in the original list.

To sort in descending order, the idioms `-@sort(-list)` or `@sort(list, -keys)` can be used.

{: #shuffle }
### `@shuffle` — Randomly shuffle a list

```
T: any
@shuffle(list: [T]): [T]
@shuffle(list: [T], seed: real): [T]

// @shuffle([1 ..= 4]) => e.g. [2, 3, 1, 4]
```

Shuffle the items in `list` using global randomness (and `seed` if it is provided).

{: #unique }
### `@unique` — Remove duplicate items in a list

```
T: any
@unique(list: [T]): [T]

// @unique([1, 4, 1, 3, 3, 2, 3, 4]) => [1, 4, 3, 2]
```

Retain only the first of each unique item in `list`. Note that this also works for items that are not normally
comparable with `==`, such as points and colors.

{: #prefix_sum }
### `@prefix_sum` — Calculate the prefix sum of a list

```
T: real | int
//  | (real | int, real | int)
//  | (real | int, real | int, real | int)
@prefix_sum(list: [T]): [T]

// @prefix_sum([]) => []
// @prefix_sum([1, 2, 3]) => [1, 1 + 2, 1 + 2 + 3] => [1, 3, 6]
// @prefix_sum([3, 2, 1]) => [3, 3 + 2, 3 + 2 + 1] => [3, 5, 6]
```

{: .note }
This intrinsic is implemented using the
[Nevin Brackett-Rozinsky O(n) Prefix Sum (Wackscope Algorithm)](https://www.desmos.com/calculator/p091kr6k84?nographpaper).

## Statistics

{: #mean }
### `@mean` — Mean/Average value

```
T: real | (real, real) | (real, real, real)
@mean(..values: T+): T+
@mean(values: [T]): T

// @mean(3, 1, 5, 2) => 2.75
// @mean([3, 1, 5, 2]) => 2.75
// @mean([]) => undefined
```

Compute the arithmetic mean/average value in `values`.

{: #median }
### `@median` — Median value

```
@median(..values: real+): real+
@median(values: [real]): real

// @median(3, 1, 5) => 3.0
// @median(3, 1, 5, 2) => 2.5
// @median([3, 1, 5, 2]) => 2.5
// @median([]) => undefined
```

Compute the median value in `values`.

{: #min }
### `@min` — Minimum value

```
T: real | int | bool
@min(..values: T+): T+
@min(values: [T]): T

// @min(3, 1, 4, 2) => 1
// @min([3, 1, 4, 2]) => 1
// @min([]) => undefined
```

Compute the minimum value in `values`. `bool` values are interpreted as 0 or 1.

{: #max }
### `@max` — Maximum value

```
T: real | int | bool
@max(..values: T+): T+
@max(values: [T]): T

// @max(3, 1, 4, 2) => 4
// @max([3, 1, 4, 2]) => 4
// @max([]) => undefined
```

Compute the maximum value in `values`. `bool` values are interpreted as 0 or 1.

{: #count }
### `@count` — Count values

```
@count(..values: any+): int+
@count(values: [any]): int

// @count(1, 2, 3) => 3
// @count([1 ..= 10]) => 10
// @count([]) => 0
```

Compute the number of values in `values`.

{: #total }
### `@total` — Total of values

```
T: real | int
//  | (real | int, real | int)
//  | (real | int, real | int, real | int)
@total(..values: T+): T+
@total(values: [T]): T

// @total(1, 2, 3) => 6
// @total([1 ..= 5]) => 15
// @total([]) => 0
```

Compute the sum of all values in `values`. `bool` values are interpreted as 0 or 1.

{: #any }
### `@any` — At least one value is `true`

```
@any(..values: bool+): bool+
@any(values: [bool]): bool

// @any(false, true, false) => true
// @any(false, false, false) => false
// @any([1 ..= 10] == 7) => true
// @any([1 ..= 10] == 11) => false
// @any([]) => false
```

Return `true` if least one input value is `true`, or `false` otherwise. This is roughly equivalent to
`@total(values) > 0`.

{: #all }
### `@all` — No values are `false`

```
@all(..values: bool+): bool+
@all(values: [bool]): bool

// @all(true, true, true) => true
// @all(true, false, true) => false
// @all([1 ..= 10] < 11) => true
// @all([1 ..= 10] < 7) => false
// @all([]) => true
```

Return `false` if at least one input value is `false` or `true` otherwise. This is roughly equivalent to
`@total(!values) == 0`.

## Visualizations

## Distributions

{: #random }
### `@random` — Generate random values in [0, 1)

```
@random(): real
@random(sample_count: int): [real]
@random(sample_count: int, seed: real): [real]

// @random() => e.g. 0.7238…
// @random(2) => e.g. [0.6329…, 0.0174…]
```

{: #choose_random }
### `@choose_random` — Choose random values from a list/distribution

```
T: any
@choose_random(list: [T]): T
@choose_random(list: [T], sample_count: int): [T]
@choose_random(list: [T], sample_count: int, seed: real): [T]
@choose_random(dist: distribution): real
@choose_random(dist: distribution, sample_count: int): [real]
@choose_random(dist: distribution, sample_count: int, seed: real): [real]

// @choose_random([1 ..= 10]) => e.g. 6
// @choose_random([1 ..= 10], 3) => e.g. [4, 10, 9]
```

## Statistical Tests

## Geometry

{: #segment }
### `@segment` — Construct a 2D line segment

```
@segment(start: (real, real)+, end: (real, real)+): segment+
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing`.

Construct a 2D line segment from `start` to `end`.

{: #segment3d }
### `@segment3d` — Construct a 3D line segment

```
@segment3d(start: (real, real, real)+, end: (real, real, real)+): segment3d+
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing`.

Construct a 3D line segment from `start` to `end`.

{: #line }
### `@line` — Construct a 2D line

```
@line(start: (real, real)+, end: (real, real)+): line+
@line(s: segment+): line+
@line(r: ray+): ray+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Construct a 2D line passing through `start` and `end`. For the versions accepting a `segment` or `ray`, the `start` and
`end` points are derived from the given object.

{: #ray }
### `@ray` — Construct a 2D line ray

```
@ray(closed_end: (real, real)+, open_end: (real, real)+): ray+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Construct a 2D line ray starting at `closed_end` and passing through `open_end`.

{: #vector }
### `@vector` — Construct a 2D vector

```
@vector(start: (real, real)+, end: (real, real)+): vector+
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing`.

Construct a 2D vector from `start` to `end`.

{: #vector3d }
### `@vector3d` — Construct a 3D vector

```
@vector3d(start: (real, real, real)+, end: (real, real, real)+): vector3d+
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing`.

Construct a 3D vector from `start` to `end`.

{: #circle }
### `@circle` — Construct a 2D circle

```
@circle(center: (real, real)+, radius: real+): circle+
@circle(center: (real, real)+, edge: (real, real)+): circle+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Construct a 2D circle centered at `center` with radius `radius` or `@distance(center, edge)`.

{: #sphere3d }
### `@sphere3d` — Construct a 3D sphere

```
@sphere3d(center: (real, real, real)+, radius: real+): sphere3d+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-graphing3d`.

Construct a 3D sphere centered at `center` with radius `radius`.

{: #arc }
### `@arc` — Construct a 2D circular arc

```
@arc(start: (real, real)+, thru: (real, real)+, end: (real, real)+): arc+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Construct a 2D circular arc from `start` to `end` which runs through `thru`.

{: #angle }
### `@angle` — Construct an undirected angle marker

```
@angle(leg_a: (real, real)+, center: (real, real)+, leg_b: (real, real)+): angle+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Construct an undirected angle marker at `center` which measures the nearest angle between `leg_a` and `leg_b`. The order
of `leg_a` and `leg_b` do not matter, and the value of the angle is always positive. Since the nearest angle is
measured, this never constructs a reflex angle.

{: #directed_angle }
### `@directed_angle` — Construct a directed angle marker

```
@directed_angle(start_leg: (real, real)+, center: (real, real)+, end_leg: (real, real)+): directed_angle+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Construct a directed angle marker at `center` which measures the nearest angle between `start_leg` and `end_leg`. The
measured angle is positive if `end_leg` is counterclockwise relative to `start_leg`, and negative if the opposite is
true. Since the smallest angle is measured, this never constructs a reflex angle.

{: #polygon }
### `@polygon` — Construct a 2D polygon

```
@polygon(..vertices: (real, real)+): polygon+
@polygon(vertices: [(real, real)]): polygon
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing3d`.

Construct a closed 2D polygon using `vertices`. If zero or one vertices are provided, the polygon is not displayed.

{: #rect }
### `@rect` — Construct a 2D rectangle

```
@rect(corner_1: (real, real), corner_2: (real, real)): polygon
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing3d`.

Construct a 2D rectangle polygon with corners at `corner_1` and `corner_2`.

{: #triangle3d }
### `@triangle3d` — Construct a 3D triangle

```
@triangle3d(a: (real, real, real)+, b: (real, real, real)+, c: (real, real, real)+): triangle3d+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-graphing3d`.

Construct a 3D triangle with vertices at `a`, `b`, and `c`.

{: #glider }
### `@glider` — Construct a geometry glider

```
T: segment | circle | line | ray | arc | polygon
@glider(object: T+, distance: real+): (real, real)+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Obtain a point along the line/curve/perimeter of `object`.
- For `segment`, `line`, `ray`, and `arc`, a distance of 0 gets the `start` point, and a distance of 1 gets the `end`
  point. Distances less than 0 and greater than 1 are clamped if the object does not continue in that direction. (For
  example, `ray` distances are clamped to the range `[0, ∞)`.)
- For `polygon`, a distance of 0 gets the first vertex, a distance of 1 gets the second vertex, and so on. Distances are
  are clamped to the range `[0, n]` where `n` is the number of vertices.

## Properties & Measurements

{: #area }
### `@area` — Area of a 2D polygon

```
@area(p: polygon+): real+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #perimeter }
### `@perimeter` — Perimeter of a 2D polygon

```
@perimeter(p: polygon+): real+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #vertices }
### `@vertices` — Vertices of a 2D polygon

```
@vertices(p: polygon): [(real, real)]
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #angles }
### `@angles` — Undirected interior angles of a 2D polygon

```
@angles(p: polygon): [angle]
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #directed_angles }
### `@directed_angles` — Directed interior angles of a 2D polygon

```
@directed_angles(p: polygon): [directed_angle]
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #segments }
### `@segments` — Segments of a 2D polygon

```
@segments(p: polygon): [segment]
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #radius }
### `@radius` — Radius of a 2D circle

```
@radius(c: circle+): real+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #center }
### `@center` — Center point of a 2D circle

```
@center(c: circle+): (real, real)+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

{: #midpoint }
### `@midpoint` — Midpoint

```
@midpoint(start: (real, real)+, end: (real, real)+): (real, real)+
@midpoint(start: (real, real, real)+, end: (real, real, real)+): (real, real, real)+
@midpoint(seg: segment+): (real, real)+
@midpoint(seg: segment3d+): (real, real, real)+
```

{: #start }
### `@start` — Start point of a vector

```
@start(v: vector+): (real, real)+
@start(v: vector3d+): (real, real, real)+
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing`.

{: #end }
### `@end` — End point of a vector

```
@end(v: vector+): (real, real)+
@end(v: vector3d+): (real, real, real)+
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing`.

## Transformations

{: #dilate }
### `@dilate` — Dilate (scale) an object about a point

```
T: (real, real) | polygon | segment | circle | arc | line | ray | vector
@dilate(object: T+, point: (real, real)+, factor: real+): T+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Dilate `object` by a factor of `factor` with `point` as the focal point.

{: #rotate }
### `@rotate` — Rotate an object about a point

```
T: (real, real) | polygon | segment | circle | arc | line | ray | vector
A: real | angle | directed_angle
@rotate(object: T+, point: (real, real)+, angle: A+): T+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Rotate `object` about `point` by `angle` (`real` angle is in radians).

{: #reflect }
### `@reflect` — Reflect an object across a line

```
T: (real, real) | polygon | segment | circle | arc | line | ray | vector
L: segment | line | ray | vector
@reflect(object: T+, line: L+): T+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Reflect `object` across `line`.

{: #translate }
### `@translate` — Translate an object

```
T: (real, real) | polygon | segment | circle | arc | line | ray | vector
@translate(object: T+, start: (real, real)+, end: (real, real)+): T+
@translate(object: T+, displacement: vector+): T+
```

{: .compatibility-note }
This intrinsic is *only* available on `--target desmos-geometry`.

Translate `object` by a given displacement (either `displacement` or the displacement from `start` to `end`).

## Color

{: #black }
### `@black` — Pure black color

```
@black: color
```

{: #white }
### `@white` — Pure white color

```
@white: color
```

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

## Advanced

{: #width_pixels }
### `@width_pixels` — Viewport width

```
@width_pixels: real
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing3d`.

The horizontal width of the current viewport, in logical pixels.

{: #height_pixels }
### `@height_pixels` — Viewport height

```
@height_pixels: real
```

{: .compatibility-note }
This intrinsic is *not* available on `--target desmos-graphing3d`.

The vertical height of the current viewport, in logical pixels.

{: #dt }
### `@dt` — Delta time between ticks

```
@dt: real
```

{: .note }
This intrinsic no longer exists. This description applies to the optional parameter to a `ticker` action.

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
This intrinsic no longer exists. This description applies to the optional parameter to a `click` attribute.

The index of the object which was clicked by the user. If the object is in a list of objects being displayed, this is
the index of that object in the list. Otherwise, `@index` is 1.

## Desmosify

{: #bool_to_internal }
### `@bool_to_internal` — Create an internal boolean value

```
@bool_to_internal(value: bool): internal_bool
```

{: .note }
This is basically useless.

{: #bool_from_internal }
### `@bool_from_internal` — Read an internal boolean value

```
@bool_from_internal(value: internal_bool): bool
```

{: .note }
This is basically useless.

{: #enum_values }
### `@enum_values` — List all valid values of an `enum` type

```
T: {some enum type}
@enum_values(enum_type: {type T}): [T]

// enum E { A, B, C }
// @enum_values(E) => [E.A, E.B, E.C]
```

{: #enum_value }
### `@enum_value` — Get the corresponding `enum` value

```
T: {some enum type}
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

{: #transparent_image_data }
### `@transparent_image_data` — Transparent image data URL

```
@transparent_image_data: str
```

The data URL provided by this intrinsic represents a 1&times;1 pixel transparent PNG image and can be used to create a
transparent image (often to provide an invisible clickable rectangle).

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

// let my_global: int = 0;
// @target_symbol(my_global) => e.g. "G_{MyGlobal}"
// action my_action() {}
// @target_symbol(action my_action) => e.g. "A_{MyAction}"
```

Given a global or action identifier, get the name of the corresponding symbol on the compilation target as a string.
This can be used to, for example, refer the user to a global or action exposed in a `public` block, or for substituting
the value of a symbol in a label.

For Desmos targets, the symbol name is the identifier converted to its LaTeX form (as shown in the above examples).
