---
layout: default
title: Online Playground
permalink: /playground
---

# {{page.title}}

<textarea id="desmosify-input" style="width: 600px; height: 400px; font-family: monospace"></textarea>

<button id="desmosify-compile-button">Compile</button>

<script src="{{site.desmos_url}}/api/v1.12/calculator.js?apiKey={{site.desmos_api_key}}"></script>

<blockquote id="desmosify-error" class="error" style="font-family: monospace; display: none">
    <pre id="desmosify-error-text"></pre>
</blockquote>

<div id="desmosify-output" style="width: 600px; height: 400px;"></div>

<script type="module">
    import init, { compile } from "{{site.url}}/playground/wasm/desmosify.js";
    await init();

    const input = document.getElementById("desmosify-input");
    const output = document.getElementById("desmosify-output");
    const error = document.getElementById("desmosify-error");
    const errorText = document.getElementById("desmosify-error-text");
    const calculator = Desmos.GraphingCalculator(output);

    document.getElementById("desmosify-compile-button").addEventListener("click", () => {
        let source = input.value;
        let graph;
        try {
            graph = JSON.parse(compile(source));
        } catch (e) {
            error.style.removeProperty("display");
            errorText.textContent = e;
            return;
        }
        error.style.display = "none";
        errorText.textContent = "";
        calculator.setState(graph);
    });
</script>
