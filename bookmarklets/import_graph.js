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
                } catch (e) {
                    alert(e);
                }
            };
            f.readAsText(i.files[0]);
        }
    });
    i.click();
})();

// (()=>{let i=document.createElement("input");i.type="file";i.accept="application/json";i.addEventListener("change",()=>{if(i.files){let f=new FileReader();f.onerror=alert;f.onload=()=>{try{Calc.setState(JSON.parse(f.result))}catch(e){alert(e)}};f.readAsText(i.files[0])}});i.click()})();
// (()%3D%3E%7Blet%20i%3Ddocument.createElement(%22input%22)%3Bi.type%3D%22file%22%3Bi.accept%3D%22application%2Fjson%22%3Bi.addEventListener(%22change%22%2C()%3D%3E%7Bif(i.files)%7Blet%20f%3Dnew%20FileReader()%3Bf.onerror%3Dalert%3Bf.onload%3D()%3D%3E%7Btry%7BCalc.setState(JSON.parse(f.result))%7Dcatch(e)%7Balert(e)%7D%7D%3Bf.readAsText(i.files%5B0%5D)%7D%7D)%3Bi.click()%7D)()%3B