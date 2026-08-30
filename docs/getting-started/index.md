---
layout: default
title: Getting Started
permalink: /getting-started
has_children: false
nav_order: 2
---

{: .no_toc }
# {{page.title}}

{: .no_toc .text-delta }
## {{site.toc_header}}

- TOC
{:toc}

## Installation

Desmosify is currently under active development, so no prepackaged releases are available yet. At the moment, the only
option is to build from source using the Rust toolchain.

## First Program

1.  Copy the following code into a new file, e.g. `hello.desmos`:
    ```
    public {
        "Hello world!";
    }
    ```
2.  Try compiling it into `hello.json` (replace `desmosify` with the path to the compiler executable):
    ```shell
    desmosify hello.desmos --out hello.json --target desmos-graphing
    ```
3.  Set up the "Import graph..." bookmarklet in your preferred browser by following the directions on the
    [Bookmarklets]({{ "/bookmarklets" | relative_url }}) page.
4.  Go to [{{site.desmos_url}}/calculator].
5.  Click on the bookmarklet, and a file dialog should appear. Navigate to `hello.json` and open it.
6.  If everything worked correctly, you should now see the text "Hello world!" in the expression list.
