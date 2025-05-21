// // import { addEventListener } from '../../packages/client'
// import { addEventListener } from '@wetspace/deskclient'
// import './index.css';

// console.log(addEventListener)

// const sumbit1 = document.getElementById('sumbit1')
// const sumbit2 = document.getElementById('sumbit2')

// const [run, removeListener] = addEventListener('privews:files', (event: any, arg: any) => {
//     console.log(event, arg, '渲染进程')
// })

// sumbit1.addEventListener('click', run)
// sumbit2.addEventListener('click', removeListener)
import { createApp } from 'vue'
import App from './App.vue'

const app = createApp(App)
app.mount('#app')