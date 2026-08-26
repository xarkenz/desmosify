---
layout: default
title: Online Playground
permalink: /playground
---

# {{page.title}}

<textarea id="desmosify-input" style="width: 600px; height: 400px; font-family: monospace"></textarea>

<button id="desmosify-compile-button">Compile</button>

<script src="{{site.desmos_url}}/api/v1.12/calculator.js?apiKey={{site.desmos_api_key}}"></script>

<p id="desmosify-error" class="error" style="font-family: monospace; display: none"></p>

<div id="desmosify-output" style="width: 600px; height: 400px;"></div>

<script type="module">
    import init, { compile } from "{{site.url}}/playground/wasm/desmosify.js";
    await init();

    const input = document.getElementById("desmosify-input");
    const output = document.getElementById("desmosify-output");
    const error = document.getElementById("desmosify-error");
    const calculator = Desmos.GraphingCalculator(output);

    document.getElementById("desmosify-compile-button").addEventListener("click", () => {
        let source = input.value;
        let graph;
        try {
            graph = JSON.parse(compile(source));
        } catch (e) {
            error.style.removeProperty("display");
            error.textContent = e;
            return;
        }
        error.style.display = "none";
        error.textContent = "";
        calculator.setState(graph);
    });
</script>
