<template>
    <div class="container">

        <div class="liveUrl">
            <div class="time">
                {{ currentTime }}
            </div>
            <SearchBar 
                ref="searchBarRef"
                @start="startListen" 
                @openWindow="openWindow" 
            />
            <el-icon :size="20" class="pushUrl" @click="dialogVisible = true">
                <Setting />
            </el-icon>
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
                :checkList="checkList"
                :pushUrl="pushUrl"
                v-model:liveInfo="liveInfo"
                @updateDiamond="updateDiamond"
            />
        </div>

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
import { ref, onMounted, onUnmounted } from 'vue'
import { LiveInfoImp } from '@/types'
import { DEFAULT_LIVE_INFO } from '@/constant'
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
import MemoList from '@/components/MemoList.vue'

const searchBarRef = ref<InstanceType<typeof SearchBar>>()
const liveVideoRef = ref<InstanceType<typeof LiveVideo>>()
const danmuListRef = ref<InstanceType<typeof DanmuList>>()

const dialogVisible = ref(false)

let socketClient: SocketCli

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
        let streamInfo: any = {}
        try { roomInfo = JSON.parse(roomJson.room_info) } catch (error) {}
        try { streamInfo = JSON.parse(roomJson.stream_info.replace(/\\"/g, '"')) } catch (error) {console.log(error) }
        console.log('roomInfo----', roomInfo)
        console.log('streamInfo----', streamInfo)

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
                const videoUrls = [
                    streamInfo?.stream?.origin?.main?.flv,
                    streamInfo?.stream?.origin?.backup?.flv,
                    roomInfo.stream_url.flv_pull_url[
                        roomInfo.stream_url.default_resolution
                    ],
                ].filter(url => url).map(url => url.replace('http://', 'https://'))
                console.log('videoUrls', videoUrls)
                
                let videoUrl = videoUrls[0]
                liveVideoRef.value?.loadLive(videoUrl, videoUrls)
                pushUrl.value = videoUrl
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
    danmuListRef.value?.clearMessages()
    socketClient?.disconnect()
}

const updateDiamond = (value: number) => {
    diamond.value = diamond.value + value
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
    const gzipData = pako.inflate(decodeMsg.payload)
    const response = douyin.Response.decode(gzipData)

    if (response.needAck) {
        const ack = douyin.PushFrame.encode({
            payloadType: 'ack',
            logId: decodeMsg.logId,
        }).finish()
        socketClient?.send(ack)
    }
    danmuListRef.value?.handleMessage(response.messagesList)
}

let timeInterval: any
const currentTime = ref(new Date().toLocaleTimeString())
const updateTime = () => {
    currentTime.value = new Date().toLocaleTimeString()
}
onMounted(() => {
    updateTime()
    timeInterval = setInterval(updateTime, 1000)
})
onUnmounted(() => {
    clearInterval(timeInterval)
})
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
    position: relative;
    .liveUrl {
        display: flex;
        flex-direction: row;
        justify-content: center;
        align-items: center;
        height: 36px;
        width: 100%;
        position: relative;
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

    .time {
        position: absolute;
        top: 50%;
        left: 3vh;
        transform: translateY(-50%);
        z-index: 999;
    }

    .pushUrl {
        position: absolute;
        top: 50%;
        right: 3vh;
        transform: translateY(-50%);
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
