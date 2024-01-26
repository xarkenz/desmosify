// Export graph...

(() => {
    let a = document.createElement("a");
    a.href = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(Calc.getState()));
    a.download = (headerController.graphsController.currentGraph.title || "untitled") + ".json";
    a.click();
})();

// (()=>{let a=document.createElement("a");a.href="data:text/json;charset=utf-8,"+encodeURIComponent(JSON.stringify(Calc.getState()));a.download=(headerController.graphsController.currentGraph.title||"untitled")+".json";a.click()})();
// (()%3D%3E%7Blet%20a%3Ddocument.createElement(%22a%22)%3Ba.href%3D%22data%3Atext%2Fjson%3Bcharset%3Dutf-8%2C%22%2BencodeURIComponent(JSON.stringify(Calc.getState()))%3Ba.download%3D(headerController.graphsController.currentGraph.title%7C%7C%22untitled%22)%2B%22.json%22%3Ba.click()%7D)()%3B