---
layout: default
title: Compiler Usage
permalink: /compiler
has_children: false
---

{: .no_toc }
# {{page.title}}

{: .no_toc .text-delta }
## {{site.toc_header}}

- TOC
{:toc}

## Command Line Syntax

```
Usage: desmosify --out <output_path> --target <target_name> [source_paths]...

Arguments:
  [source_paths]...  The paths of source code files to compile into a single program

Options:
  -o, --out <output_path>     The path where compilation output will be written to
  -t, --target <target_name>  The name of the compilation target
  -h, --help                  Print help
  -V, --version               Print version
```

## Compilation Targets

The current list of compilation targets is as follows:

- `desmos-graphing` — [Desmos Graphing Calculator]({{site.desmos_url}}/calculator)
- `desmos-geometry` — [Desmos Geometry Calculator]({{site.desmos_url}}/geometry)
- `desmos-graphing3d` — [Desmos 3D Graphing Calculator]({{site.desmos_url}}/3d) (not yet supported)
