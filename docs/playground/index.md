---
layout: default
title: Online Playground
permalink: /playground
---

# {{page.title}}

<textarea id="desmosify-input" style="width: 600px; height: 400px; font-family: monospace"></textarea>

<button id="desmosify-compile-button">Compile</button>

<blockquote id="desmosify-error" class="error" style="display: none">
    <pre id="desmosify-error-text"></pre>
</blockquote>

<div id="desmosify-output" style="width: 600px; height: 400px;"></div>

<script src="{{site.desmos_url}}/api/v1.12/calculator.js?apiKey={{site.desmos_api_key}}"></script>

<script type="module">
    import init, { compile } from '{{ "/playground/wasm/desmosify.js" | relative_url }}';
    await init();

    const input = document.getElementById("desmosify-input");
    const output = document.getElementById("desmosify-output");
    const error = document.getElementById("desmosify-error");
    const errorText = document.getElementById("desmosify-error-text");
    const calculator = Desmos.GraphingCalculator(output);

    document.getElementById("desmosify-compile-button").addEventListener("click", () => {
        let source = input.value;
        let rawOutput, graph;
        try {
            rawOutput = compile(source);
            graph = JSON.parse(rawOutput);
        } catch (e) {
            error.style.removeProperty("display");
            errorText.textContent = e;
            console.error({ error: e, source, rawOutput });
            return;
        }
        error.style.display = "none";
        errorText.textContent = "";
        calculator.setState(graph);
    });

    fetch('{{ "/playground/examples/fibonacci.desmos" | relative_url }}')
        .then(response => response.text())
        .then(text => {
            input.value = text;
        });
</script>
