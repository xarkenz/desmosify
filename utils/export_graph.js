// Export graph...

(() => {
    if (!Calc) return alert("This action is only allowed on a Desmos page.");
    let a = document.createElement("a");
    a.href = "data:application/json;charset=utf-8," + encodeURIComponent(JSON.stringify(Calc.getState()));
    a.download = (document.getElementById("dcg-graph-title-text")?.textContent || "untitled") + ".json";
    a.click();
})();

// (()=>{if(!Calc)return alert("This action is only allowed on a Desmos page.");let a=document.createElement("a");a.href="data:application/json;charset=utf-8,"+encodeURIComponent(JSON.stringify(Calc.getState()));a.download=(document.getElementById("dcg-graph-title-text")?.textContent||"untitled")+".json";a.click()})();
// (()%3D%3E%7Bif(!Calc)return%20alert(%22This%20action%20is%20only%20allowed%20on%20a%20Desmos%20page.%22)%3Blet%20a%3Ddocument.createElement(%22a%22)%3Ba.href%3D%22data%3Aapplication%2Fjson%3Bcharset%3Dutf-8%2C%22%2BencodeURIComponent(JSON.stringify(Calc.getState()))%3Ba.download%3D(document.getElementById(%22dcg-graph-title-text%22)%3F.textContent%7C%7C%22untitled%22)%2B%22.json%22%3Ba.click()%7D)()%3B
