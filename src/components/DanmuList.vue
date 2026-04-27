<template>
    <DynamicScroller
        :items="messageList"
        :min-item-size="32"
        class="liveMeg"
        id="liveMsg"
        @scroll="handleScroll"
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
import { douyin } from '@/proto/dy.js'
import { DEFAULT_MESSAGE } from '@/constant'

export interface Message {
    id: string
    name: string
    msg: string
}

export interface DanmuListExpose {
    addMessage: (msg: Message) => void
    clearMessages: () => void
    handleMessage: (messagesList: douyin.Message[]) => void
}

const props = defineProps<{
    checkList?: string[]
    pushUrl?: string
}>()

const liveInfo = defineModel<any>()

const emit = defineEmits<{
    updateDiamond: [value: number]
}>()

const messageList = ref<Message[]>([...DEFAULT_MESSAGE])

let scrollTimer: ReturnType<typeof setTimeout> | null = null

const scrollToBottom = () => {
    const scroller = document.getElementById('liveMsg')
    if (scroller) {
        scroller.scrollTop = scroller.scrollHeight + 999
    }
}

const handleScroll = () => {
    if (scrollTimer) clearTimeout(scrollTimer)
    scrollTimer = setTimeout(() => {
        scrollTimer = null
    }, 2000)
}

const addMessage = (message: Message) => {
    messageList.value.push(message)
    if (!scrollTimer) scrollToBottom()
}

const clearMessages = () => {
    messageList.value = [...DEFAULT_MESSAGE]
}

const pushMessage = async (msg: { type: string; data: any }) => {
    if (!props.pushUrl) return
    try {
        await fetch(props.pushUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(msg),
        })
    } catch (error) {
        console.error('push message error:', error)
    }
}

const handleMessage = (messageList: douyin.Message[]) => {
    messageList.forEach((msg) => {
        switch (msg.method) {
            case 'WebcastMatchAgainstScoreMessage':
                break
            case 'WebcastLikeMessage':
                likeLive(msg.payload)
                break
            case 'WebcastMemberMessage':
                enterLive(msg.payload)
                break
            case 'WebcastGiftMessage':
                decodeGift(msg.payload)
                break
            case 'WebcastChatMessage':
                decodeChat(msg.payload)
                break
            case 'WebcastSocialMessage':
                followLive(msg.payload)
                break
            case 'WebcastUpdateFanTicketMessage':
                break
            case 'WebcastCommonTextMessage':
                break
            case 'WebcastProductChangeMessage':
                break
            case 'WebcastRoomUserSeqMessage':
                countLive(msg.payload)
                break
            default:
                console.log('待解析方法' + msg.method)
                break
        }
    })
}

const decodeChat = (data: any) => {
    const chatMsg = douyin.ChatMessage.decode(data)
    const { common, user, content } = chatMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: content,
    }
    props.checkList?.includes('chat') && addMessage(message)
    pushMessage({ type: 'chat', data: message })
}

const decodeGift = (data: any) => {
    const giftMsg = douyin.GiftMessage.decode(data)
    const { common, user, gift, repeatCount } = giftMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: `送出${gift.name} x${repeatCount}个`,
    }
    props.checkList?.includes('gift') && addMessage(message)
    emit('updateDiamond', gift.diamondCount * repeatCount)
    pushMessage({ type: 'gift', data: message })
}

const enterLive = (data: any) => {
    const enteryMsg = douyin.MemberMessage.decode(data)
    const { common, user } = enteryMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: '来了',
    }
    props.checkList?.includes('comein') && addMessage(message)
    pushMessage({ type: 'comein', data: message })
}

const likeLive = (data: any) => {
    const likeMsg = douyin.LikeMessage.decode(data)
    const { common, user, total } = likeMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: `为主播点赞了`,
    }
    liveInfo.value && (liveInfo.value.totalLike = total)
    props.checkList?.includes('like') && addMessage(message)
    pushMessage({ type: 'like', data: message })
}

const followLive = (data: any) => {
    const followMsg = douyin.SocialMessage.decode(data)
    const { common, user, followCount } = followMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: `关注了主播`,
    }
    liveInfo.value && (liveInfo.value.fans = followCount)
    props.checkList?.includes('follow') && addMessage(message)
    pushMessage({ type: 'follow', data: message })
}

const countLive = (data: any) => {
    const countMsg = douyin.RoomUserSeqMessage.decode(data)
    liveInfo.value && (liveInfo.value.customer = countMsg.onlineUserForAnchor)
}

defineExpose({
    addMessage,
    clearMessages,
    handleMessage,
    scrollToBottom,
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
