<template>
    <div>主页</div>
    <button @click="run">开始</button>
    <button @click="remove">移除</button>
</template>
<script lang="ts" setup>
import{ addEventListener } from '@wetspace/deskclient'

const [run,remove] = addEventListener('privews:files', (e,v) => {
    console.log('页面加载',e,v)
})

console.log(addEventListener)
</script>