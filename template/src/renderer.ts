import { addListener,removeListener } from '../../packages/client'
import './index.css';

const sumbit1 = document.getElementById('sumbit1')
const sumbit2 = document.getElementById('sumbit2')
// const fn = (v,c)=>{
//     console.log(v,c)
// }
function fn(event:any,arg:any){
    console.log(event,arg)
}

function fn1(event:any,arg:any){
    console.log(event,arg,'fn1')
}
sumbit1.addEventListener('click',()=>{
    console.log(addListener)
    addListener('privews:files',fn)
    addListener('privews:files',fn1)
    // window.electronEvents.addListener('privews:files',fn)
}) 
sumbit2.addEventListener('click',()=>{
    removeListener('privews:files',fn)
    removeListener('privews:files',fn1)
    // window.electronEvents.removeListener('privews:files',fn)
})