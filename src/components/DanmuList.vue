<template>
    <DynamicScroller
        :items="messageList"
        :min-item-size="32"
        class="liveMeg"
        id="liveMsg"
        ref="liveMsg"
        v-if="messageList.length"
    >
        <template v-slot="{ item, active }">
            <DynamicScrollerItem
                :item="item"
                :active="active"
                class="msgBox"
                :size-dependencies="[item.name, item.msg]"
                :data-index="item.id"
            >
                <div class="content">
                    <span class="name">{{ item.name }}：</span>
                    <span class="msg">{{ item.msg }}</span>
                </div>
            </DynamicScrollerItem>
        </template>
    </DynamicScroller>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { DynamicScroller, DynamicScrollerItem } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

export interface Message {
    id: string
    name: string
    msg: string
}

defineProps<{
    messageList: Message[]
}>()

const liveMsg = ref()

defineExpose({
    liveMsg
})
</script>

<style scoped lang="scss">
.liveMeg {
    flex: 1;
    margin-left: 10px;
    background-color: #252632;
    border-radius: 10px;
    box-shadow: 0 0 10px 2px gray;
    scrollbar-color: #363741 transparent;
    scrollbar-width: thin;
    overflow-y: scroll;

    .msgBox {
        display: flex;
        flex-direction: row;
        padding: 5px;
        white-space: wrap;

        .name {
            color: #8ce7ff;
            margin-right: 2px;
            white-space: nowrap;
        }

        .msg {
            color: white;
            white-space: wrap;
        }
    }
}
</style>
