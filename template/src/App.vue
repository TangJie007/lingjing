<template>
    <div>主页</div>
    <button @click="listen">开始</button>
    <button @click="remove">移除</button>
</template>
<script lang="ts" setup>
import{ addEventListener } from '@wetspace/deskclient'

const [listen,remove] = addEventListener('privews:files', (e:string) => {
    console.log('页面加载',e)
})
</script>