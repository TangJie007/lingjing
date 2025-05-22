<template>
主页
<button @click="openModal">打开弹窗</button>
</template>
<script setup lang="ts">
import { invokeEvents } from '@wetspace/deskclient'
const openModal = async ()=>{
    const res = await invokeEvents('create:dialog','win')
    console.log(res,'渲染进程')
}
</script>