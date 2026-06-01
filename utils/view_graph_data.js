// Open graph data in new tab

(() => {
    if (!Calc) return alert("This action is only allowed on a Desmos page.");
    window.open("data:application/json;charset=utf-8," + encodeURIComponent(JSON.stringify(Calc.getState())), "_blank");
})();

// (()=>{if(!Calc)return alert("This action is only allowed on a Desmos page.");window.open("data:application/json;charset=utf-8,"+encodeURIComponent(JSON.stringify(Calc.getState())),"_blank")})();
