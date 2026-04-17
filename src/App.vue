<template>
    <div class="container">
        <div class="liveUrl">
            <SearchBar 
                ref="searchBarRef"
                @start="startListen" 
                @openWindow="openWindow" 
            />
        </div>

        <div class="liveBox">
            <LiveVideo 
                ref="liveVideoRef"
                :liveInfo="liveInfo" 
                :diamond="diamond"
                @pay="handlePay"
            />
            <DanmuList 
                ref="danmuListRef"
                :messageList="messageList" 
            />
        </div>

        <el-icon :size="20" class="pushUrl" @click="dialogVisible = true">
            <Setting />
        </el-icon>

        <MemoList :liveInfo="liveInfo" />
    </div>

    <el-dialog
        v-model="dialogVisible"
        title="设置推送地址"
        center
        :show-close="false"
        width="540"
    >
        <div class="setBox">
            <el-input v-model="pushUrl" placeholder="请输入推送地址" />
            <div class="messageSel">
                <span>选择消息类型：</span>
                <el-checkbox-group v-model="checkList">
                    <el-checkbox label="聊天" value="chat" />
                    <el-checkbox label="礼物" value="gift" />
                    <el-checkbox label="点赞" value="like" />
                    <el-checkbox label="关注" value="follow" />
                    <el-checkbox label="进来" value="comein" />
                </el-checkbox-group>
            </div>
            <div class="messageSel">
                <span>直播录制配置：</span>
                <el-checkbox-group v-model="recordVideo">
                    <el-checkbox label="开启录制" value="open" />
                    <el-checkbox label="录制弹幕" value="chat" />
                    <el-checkbox label="录制礼物" value="gift" />
                </el-checkbox-group>
            </div>
            <div class="tips">
                *推送的消息会以POST请求的形式发送到该地址，请确保该地址能够接收POST请求
            </div>
        </div>
        <template #footer>
            <div class="dialog-footer">
                <el-button @click="dialogVisible = false">取消</el-button>
                <el-button type="primary" @click="savePushUrl">
                    确定
                </el-button>
            </div>
        </template>
    </el-dialog>
</template>

<script setup lang="ts">
import { Setting } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ref } from 'vue'
import { LiveInfoImp } from '@/types'
import Logo from '@/assets/logo.png'
import { ConnectionConfig } from 'tauri-plugin-websocket-api'
import { douyin } from '@/proto/dy.js'
import { ElMessage } from 'element-plus'
import SocketCli from '@/utils/RustSocket'
import { emit } from '@tauri-apps/api/event'
import pako from 'pako'
import SearchBar from '@/components/SearchBar.vue'
import LiveVideo from '@/components/LiveVideo.vue'
import type { LiveInfo } from '@/components/LiveVideo.vue'
import DanmuList from '@/components/DanmuList.vue'
import type { Message } from '@/components/DanmuList.vue'
import MemoList from '@/components/MemoList.vue'

const searchBarRef = ref<InstanceType<typeof SearchBar>>()
const liveVideoRef = ref<InstanceType<typeof LiveVideo>>()
const danmuListRef = ref<InstanceType<typeof DanmuList>>()

const dialogVisible = ref(false)

const messageList = ref<Message[]>([
    {
        id: '1',
        name: '管理员',
        msg: '欢迎使用直播盒子，输入直播地址开始安静看直播，没有刷礼物功能，所以理性看播，不要乱消费',
    },
])

let socketClient: SocketCli

const DEFAULT_LIVE_INFO = {
    uid: '888888',
    status: 0,
    title: '直播标题',
    name: 'Livebox',
    roomId: '888888',
    avatar: Logo,
    fans: 0,
    customer: 0,
    totalLike: 0,
    signature: '',
}

const liveInfo = ref<LiveInfo>({ ...DEFAULT_LIVE_INFO })

const diamond = ref(0)
const pushUrl = ref(localStorage.getItem('pushUrl') || '')
const checkList = ref<string[]>(['chat'])
const recordVideo = ref<string[]>([])

let liveNum = 100

const savePushUrl = () => {
    localStorage.setItem('pushUrl', pushUrl.value)
    dialogVisible.value = false
}

const openWindow = (url: string) => {
    invoke('open_window', {
        appUrl: url,
        appName: '直播盒子' + liveNum++,
        platform: 'web',
        userAgent: navigator.userAgent,
        resize: false,
        width: 1000,
        height: 800,
        jsContent: '',
    })
}

const handlePay = () => {
    console.log('emit handlepay')
    emit('handlepay')
}

const startListen = async (inputUrl: string) => {
    const url = inputUrl.trim()
    localStorage.setItem('url', url)
    clearLivex()
    if (url.trim()) {
        const urlPath = url.split('?')[0];
		if (urlPath.endsWith('.flv') || urlPath.endsWith('.m3u8')) {
            liveInfo.value = {
                ...DEFAULT_LIVE_INFO,
                title: '直链播放',
            }
            searchBarRef.value?.saveToHistory({
                title: '直链播放',
                anchor_name: 'Livebox',
                roomId: '888888',
                url: url,
            })
            liveVideoRef.value?.loadLive(url)
            return
        }

        const roomJson: LiveInfoImp = await invoke('get_live_html', { url })
        let roomInfo: any = {}
        try { roomInfo = JSON.parse(roomJson.room_info) } catch (error) {}
        console.log('roomInfo----', roomInfo)

        if (roomInfo.id_str) {
            if (roomInfo.status) {
                ElMessage.success('open live success!')
                liveInfo.value = {
                    uid: roomInfo.owner.id_str,
                    status: roomInfo.status,
                    title: roomInfo.title,
                    name: roomInfo.owner.nickname,
                    roomId: roomInfo.id_str,
                    avatar: roomInfo.owner.avatar_thumb.url_list[0],
                    fans: 0,
                    customer: roomInfo.user_count_str,
                    totalLike: roomInfo.stats.total_user_str,
                    signature: 'roomInfo.signature',
                }
                searchBarRef.value?.saveToHistory({
                    title: roomInfo.title,
                    anchor_name: roomInfo.owner.nickname,
                    roomId: url.match(/\d{9,12}/g)?.shift() || roomInfo.id_str,
                    url: url,
                })
                let videoUrl = roomInfo.stream_url.flv_pull_url[
                    roomInfo.stream_url.default_resolution
                ].replace('http://', 'https://')
                liveVideoRef.value?.loadLive(videoUrl)
                creatSokcet(roomInfo.id_str, roomJson.unique_id, roomJson.ttwid)
            } else {
                ElMessage.success('live is over!')
                liveInfo.value = {
                    uid: roomInfo.id_str,
                    status: 4,
                    title: '已停播',
                    name: roomInfo.nickname,
                    roomId: roomInfo.id_str,
                    avatar: roomInfo.avatar_thumb.url_list[0],
                    fans: 0,
                    customer: 0,
                    totalLike: 0,
                    signature: 'roomInfo.signature',
                }
                liveVideoRef.value?.destroyPlayer()
            }
        } else {
            console.log('没有获取到')
            ElMessage.error('open live error')
        }
    }
}

const clearLivex = () => {
    liveVideoRef.value?.destroyPlayer()
    messageList.value = [
        {
            id: '1',
            name: '管理员',
            msg: '欢迎使用直播盒子，输入直播地址开始安静看直播，没有刷礼物功能，所以理性看播，不要乱消费',
        },
    ]
    socketClient?.disconnect()
}

const creatSokcet = async (roomId: string, uniqueId: string, ttwid: string) => {
    let sign = window.creatSignature(roomId, uniqueId)
    let socketUrl = `wss://webcast5-ws-web-lf.douyin.com/webcast/im/push/v2/?room_id=${roomId}&compress=gzip&version_code=180800&webcast_sdk_version=1.0.14-beta.0&live_id=1&did_rule=3&user_unique_id=${uniqueId}&identity=audience&signature=${sign}&aid=6383&device_platform=web&browser_language=zh-CN&browser_platform=Win32&browser_name=Mozilla&browser_version=5.0+%28Windows+NT+10.0%3B+Win64%3B+x64%29+AppleWebKit%2F537.36+%28KHTML%2C+like+Gecko%29+Chrome%2F126.0.0.0+Safari%2F537.36+Edg%2F126.0.0.0`
    const options: ConnectionConfig = {
        writeBufferSize: 20000,
        headers: {
            cookie: 'ttwid=' + ttwid,
            'user-agent':
                'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0',
        },
    }
    const pingMsg = douyin.PushFrame.encode({ payloadType: 'hb' }).finish()
    socketClient = new SocketCli(socketUrl, options, onMessage, pingMsg)
}

const onMessage = (msg: any) => {
    const decodeMsg = douyin.PushFrame.decode(msg.data)
    if (danmuListRef.value) {
        const msgDom: HTMLElement | null = document.getElementById('liveMsg')
        if (msgDom) {
            msgDom.scrollTop = msgDom.scrollHeight
        }
    }
    const gzipData = pako.inflate(decodeMsg.payload)
    const response = douyin.Response.decode(gzipData)

    if (response.needAck) {
        const ack = douyin.PushFrame.encode({
            payloadType: 'ack',
            logId: decodeMsg.logId,
        }).finish()
        socketClient?.send(ack)
    }
    handleMessage(response.messagesList)
}

const handleMessage = (messageList: douyin.Message) => {
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
    checkList.value.includes('chat') && messageList.value.push(message)
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
    checkList.value.includes('gift') && messageList.value.push(message)
    diamond.value = diamond.value + gift.diamondCount * repeatCount
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
    checkList.value.includes('comein') && messageList.value.push(message)
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
    liveInfo.value = {
        ...liveInfo.value,
        totalLike: total,
    }
    checkList.value.includes('like') && messageList.value.push(message)
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
    liveInfo.value = {
        ...liveInfo.value,
        fans: followCount,
    }
    checkList.value.includes('follow') && messageList.value.push(message)
    pushMessage({ type: 'follow', data: message })
}

const countLive = (data: any) => {
    const countMsg = douyin.RoomUserSeqMessage.decode(data)
    liveInfo.value = {
        ...liveInfo.value,
        customer: countMsg.onlineUserForAnchor,
    }
}

const pushMessage = async (msg: { type: string; data: any }) => {
    if (!pushUrl.value) return
    try {
        await fetch(pushUrl.value, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(msg),
        })
    } catch (error) {
        console.error('push message error:', error)
    }
}
</script>

<style scoped lang="scss">
.container {
    width: 100%;
    height: 100%;
    padding: 20px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    background-color: #f5f5f5;
    .liveUrl {
        display: flex;
        flex-direction: row;
        justify-content: center;
        align-items: center;
        height: 36px;
        width: 100%;
    }

    .liveBox {
        flex: 1;
        display: flex;
        width: 100%;
        height: calc(100% - 20px);
        padding: 20px;
        flex-direction: row;
        justify-content: center;
    }

    .pushUrl {
        position: fixed;
        top: 3vh;
        right: 3vh;
        z-index: 999;
    }
}

.setBox {
    margin: 2vh 20px;

    .messageSel {
        margin-top: 4px;
    }

    .tips {
        font-size: small;
        color: #999;
    }
}
</style>
