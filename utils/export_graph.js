// Export graph...

(() => {
    if (!Calc) return alert("This action is only allowed on a Desmos page.");
    let a = document.createElement("a");
    a.href = "data:application/json;charset=utf-8," + encodeURIComponent(JSON.stringify(Calc.getState()));
    a.download = (document.getElementById("dcg-graph-title-text")?.textContent || "untitled") + ".json";
    a.click();
})();

// (()=>{if(!Calc)return alert("This action is only allowed on a Desmos page.");let a=document.createElement("a");a.href="data:application/json;charset=utf-8,"+encodeURIComponent(JSON.stringify(Calc.getState()));a.download=(document.getElementById("dcg-graph-title-text")?.textContent||"untitled")+".json";a.click()})();
