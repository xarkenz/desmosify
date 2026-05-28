// Open graph data in new tab

(() => {
    if (!Calc) return alert("This action is only allowed on a Desmos page.");
    window.open("data:application/json;charset=utf-8," + encodeURIComponent(JSON.stringify(Calc.getState())), "_blank");
})();

// (()=>{if(!Calc)return alert("This action is only allowed on a Desmos page.");window.open("data:application/json;charset=utf-8,"+encodeURIComponent(JSON.stringify(Calc.getState())),"_blank")})();
// (()%3D%3E%7Bif(!Calc)return%20alert(%22This%20action%20is%20only%20allowed%20on%20a%20Desmos%20page.%22)%3Bwindow.open(%22data%3Aapplication%2Fjson%3Bcharset%3Dutf-8%2C%22%2BencodeURIComponent(JSON.stringify(Calc.getState()))%2C%22_blank%22)%7D)()%3B
