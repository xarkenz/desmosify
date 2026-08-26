---
layout: default
title: Online Playground
permalink: /playground
---

# {{page.title}}

<script type="module">
    import init, { compile } from "{{site.url}}/playground/wasm/desmosify.js";
    await init();
    console.log(compile("hello world!"));
</script>
