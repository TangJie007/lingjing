import './index.css';

console.log(window)
const sumbit1 = document.getElementById('sumbit1')
const sumbit2 = document.getElementById('sumbit2')
// const fn = (v,c)=>{
//     console.log(v,c)
// }
function fn(v){
    console.log(v)
}
sumbit1.addEventListener('click',()=>{
    // console.log(window.electronEvents)
    window.electronEvents.addListener('privews:files',fn)
})