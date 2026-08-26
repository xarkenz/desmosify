---
layout: default
title: Online Playground
permalink: /playground
---

# {{page.title}}

<textarea id="desmosify-input"></textarea>

<button id="desmosify-compile-button">Compile</button>

<script type="module">
    import init, { compile } from "{{site.url}}/playground/wasm/desmosify.js";
    await init();

    const inputTextArea = document.getElementById("desmosify-input");
    document.getElementById("desmosify-compile-button").addEventListener("click", () => {
        let source = inputTextArea.textContent;
        console.log(compile(source));
    });
</script>
