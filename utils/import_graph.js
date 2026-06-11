// Import graph...

(() => {
    let i = document.createElement("input");
    i.type = "file";
    i.accept = "application/json";
    i.addEventListener("change", () => {
        if (i.files) {
            let f = new FileReader();
            f.onerror = alert;
            f.onload = () => {
                try {
                    Calc.setState(JSON.parse(f.result));
                    Calc.controller._hasUnsavedChanges = true;
                } catch (e) {
                    alert(e);
                }
            };
            f.readAsText(i.files[0]);
        }
    });
    i.click();
})();

// (()=>{let i=document.createElement("input");i.type="file";i.accept="application/json";i.addEventListener("change",()=>{if(i.files){let f=new FileReader();f.onerror=alert;f.onload=()=>{try{Calc.setState(JSON.parse(f.result));Calc.controller._hasUnsavedChanges=true}catch(e){alert(e)}};f.readAsText(i.files[0])}});i.click()})();
