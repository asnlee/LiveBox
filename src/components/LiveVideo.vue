<template>
    <div class="liveVideo">
        <div class="ownerBox">
            <img :src="liveInfo.avatar" alt="头像" class="avatar" />
            <div class="nickBox">
                <span class="nickName">{{ liveInfo.name }}</span>
                <span class="fans">
                    {{ liveInfo.totalLike }}本场点赞
                </span>
            </div>
        </div>
        <div class="likeInfo">
            <div class="customer">
                观众：{{ liveInfo.customer }}
            </div>
            <!-- <div class="fans" @click="handlePay">
                粉丝：{{ liveInfo.fans }}
            </div>
            <div class="diamond">收益：{{ diamond }}</div> -->
        </div>
        <div v-if="dplayer" class="searchSame" @click.stop="handleSearchSame" :style="{opacity: loading ? 0.7 : 1}">
            <el-icon :class="{ 'is-loading': loading }">
                <Loading v-if="loading" />
                <Search v-else />
            </el-icon>
            搜同款
        </div>
        <div id="dplayer" class="dplayer"></div>
        <div v-if="liveInfo.status === 4" class="over">直播已结束</div>
    </div>
</template>

<script setup lang="ts">
import { Search, Loading } from '@element-plus/icons-vue'
import DPlayer from 'dplayer'
import Hls from 'hls.js'
import Flv from 'flv.js'
import { writeBinaryFile } from '@tauri-apps/api/fs';
import { dialog } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/tauri'
import { ref } from 'vue';

export interface LiveInfo {
    uid: string
    status: number
    title: string
    name: string
    roomId: string
    avatar: string
    fans: number
    customer: number
    totalLike: number
    signature: string
}

const props = defineProps<{
    liveInfo: LiveInfo
    diamond: number
}>()

const emit = defineEmits<{
    (e: 'pay'): void
}>()

let dplayer: DPlayer | null = null
let hls: Hls | null = null
let flvPlayer: any = null
let latencyTimer: ReturnType<typeof setInterval> | null = null

const handlePay = () => {
    emit('pay')
}

const startLatencyMonitor = () => {
    stopLatencyMonitor()
    latencyTimer = setInterval(() => {
        const video = dplayer?.video
        if (!video || video.paused || !video.buffered.length) return

        const end = video.buffered.end(video.buffered.length - 1)
        const latency = end - video.currentTime

        if (latency > 1.5) {
            video.currentTime = end - 0.1
        } else if (latency > 0.5) {
            if (video.playbackRate === 1.0) video.playbackRate = 1.1
        } else {
            if (video.playbackRate !== 1.0) video.playbackRate = 1.0
        }
    }, 1000)
}

const stopLatencyMonitor = () => {
    if (latencyTimer) {
        clearInterval(latencyTimer)
        latencyTimer = null
    }
    if (dplayer?.video && dplayer.video.playbackRate === 1.1) {
        dplayer.video.playbackRate = 1.0
    }
}

const loadLive = (videoUrl: string, live: boolean = true) => {
    stopLatencyMonitor()
    if (dplayer) {
        dplayer.destroy()
        dplayer = null
    }
    hls = null
    flvPlayer = null

    if (videoUrl.includes('m3u8')) {
        dplayer = new DPlayer({
            container: document.getElementById('dplayer'),
            screenshot: true,
            autoplay: true,
            live: live,
            lang: 'zh-cn',
            video: {
                url: '',
                type: 'customHls',
                customType: {
                    customHls: function (video: any, _: any) {
                        hls = new Hls({
                            liveSyncDuration: 1,
                            liveMaxLatencyDuration: 2,
                            liveDurationInfinity: true,
                            maxBufferLength: 5,
                            maxMaxBufferLength: 10,
                        })
                        hls.loadSource(videoUrl)
                        hls.attachMedia(video)
                        if (live) startLatencyMonitor()
                    },
                },
            },
        })
    } else if (videoUrl.includes('mp4')) {
        dplayer = new DPlayer({
            container: document.getElementById('dplayer'),
            live: live,
            autoplay: true,
            screenshot: true,
            fullScreen: false,
            lang: 'zh-cn',
            video: {
                url: videoUrl,
                type: 'mp4',
            },
        })
    } else if (videoUrl.includes('flv')) {
        dplayer = new DPlayer({
            container: document.getElementById('dplayer'),
            screenshot: true,
            live: live,
            autoplay: true,
            lang: 'zh-cn',
            video: {
                url: videoUrl,
                type: 'customFlv',
                customType: {
                    customFlv: function (video: any, _: any) {
                        flvPlayer = Flv.createPlayer({
                            type: 'flv',
                            url: videoUrl,
                            hasAudio: true,
                            hasVideo: true
                        }, {
                            isLive: true,
                            enableStashBuffer: false,
                            lazyLoad: false,
                            accurateSeek: false,
                            fixAudioTimestampGap: true
                        })
                        flvPlayer.attachMediaElement(video)
                        flvPlayer.load()
                        if (live) startLatencyMonitor()
                    },
                },
            },
        })
    }

    dplayer.on('screenshot', () => {
        captureScreenshot(dplayer.video)
    })
}

const saveScreenshot = async (link: HTMLAnchorElement) => {
    const base64Data = link.href.split(',')[1]
    const time = new Date().toLocaleString('zh-CN').replace(/[\/\s:]/g, '-')
    const fileName = `${props.liveInfo.name}(${time}).png`

    try {
        const binaryString = atob(base64Data)
        const bytes = new Uint8Array(binaryString.length)
        for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i)
        }

        const filePath = await dialog.save({
            defaultPath: fileName,
            filters: [{ name: 'Images', extensions: ['png'] }]
        })

        if (filePath) {
            await writeBinaryFile(filePath, bytes)
        }
    } catch (e) {
        alert('保存截图失败:' + e)
        console.error('保存截图失败:', e)
    }
}

const captureScreenshot = (video: HTMLVideoElement) => {
    if (!video) return

    const canvas = document.createElement('canvas')
    canvas.width = video.videoWidth
    canvas.height = video.videoHeight
    const ctx = canvas.getContext('2d') as CanvasRenderingContext2D
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height)

    const link = document.createElement('a')
    const time = new Date().toLocaleString('zh-CN').replace(/[\/\s:]/g, '-')
    link.download = `${props.liveInfo.name}(${time}).png`
    link.href = canvas.toDataURL('image/png')
    link.click()
    
    // Save to file using Tauri fs plugin
    if (window.__TAURI__){
        saveScreenshot(link)
    }
}

const loading = ref(false)
const handleSearchSame = async () => {
    const video = dplayer?.video
    if (!video || loading.value) return

    let width = video.videoWidth
    let height = video.videoHeight
    const maxSize = 720
    if (width > maxSize || height > maxSize) {
        const ratio = Math.min(maxSize / width, maxSize / height)
        width = Math.floor(width * ratio)
        height = Math.floor(height * ratio)
    }

    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    const ctx = canvas.getContext('2d') as CanvasRenderingContext2D
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height)

    canvas.toBlob(async (blob) => {
        if (!blob) return
        const arrayBuffer = await blob.arrayBuffer()
        const bytes = Array.from(new Uint8Array(arrayBuffer))
        const name = `live-screenshot.jpg`

        try {
            loading.value = true
            const imageUrl = await invoke<string>('upload_file', { fileName: name, fileData: bytes })
            const baiduUrl = 'https://graph.baidu.com/details?isfromtusoupc=1&tn=pc&carousel=0&promotion_name=pc_image_shituindex&extUiData%5bisLogoShow%5d=1&image=' + imageUrl
            invoke('open_window', {
                appUrl: baiduUrl,
                appName: '搜同款',
                platform: 'web',
                userAgent: navigator.userAgent,
                resize: false,
                width: 1000,
                height: 800,
                jsContent: '',
            })
        } catch (e) {
            console.log('上传失败')
        } finally {
            loading.value = false
        }
    }, 'image/jpeg', 0.8)
}

const destroyPlayer = () => {
    stopLatencyMonitor()
    if (hls) {
        hls.destroy()
        hls = null
    }
    if (flvPlayer) {
        flvPlayer.destroy()
        flvPlayer = null
    }
    if (dplayer) {
        dplayer.destroy()
        dplayer = null
    }
}

defineExpose({
    loadLive,
    destroyPlayer
})
</script>

<style scoped lang="scss">
.liveVideo {
    position: relative;
    width: 72%;
    height: 100%;
    border-radius: 10px;
    background-image: url('@/assets/images/liveBg.webp');
    background-position: center;
    background-size: cover;
    background-repeat: no-repeat;
    background-color: rgba(0, 0, 0, 0.5);
    box-shadow: 0 0 10px 2px gray;
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;

    .ownerBox {
        position: absolute;
        top: 10px;
        left: 10px;
        height: 40px;
        display: flex;
        flex-direction: row;
        align-items: center;
        background-color: #0000008b;
        padding: 10px 4px;
        border-radius: 20px;
        z-index: 999;
        user-select: none;

        .avatar {
            width: 32px;
            height: 32px;
            border-radius: 50%;
            margin-right: 5px;
        }

        .nickBox {
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: flex-start;
            margin-right: 10px;

            .nickName {
                font-size: 14px;
                color: white;
                user-select: none;
            }

            .fans {
                font-size: 11px;
                color: #ccc;
                user-select: none;
            }
        }
    }

    .likeInfo {
        position: absolute;
        top: 10px;
        right: 10px;
        height: 40px;
        display: flex;
        flex-direction: row;
        align-items: center;
        padding: 10px 4px;
        border-radius: 20px;
        z-index: 999;
        user-select: none;
        color: white;
    }

    .searchSame {
        position: absolute;
        top: 50%;
        right: 10px;
        transform: translateY(-50%);
        color: white;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 5px;
        z-index: 999;
        letter-spacing: 2px;
        font-size: 14px;
    }

    .dplayer {
        width: 100%;
        height: 100%;
        border-radius: 10px;
    }

    .over {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        display: flex;
        flex-direction: row;
        justify-content: center;
        align-items: center;
        color: white;
        font-size: 25px;
        font-weight: bold;
        user-select: none;
        text-shadow: 0 0 6px 2px black;
    }
}
</style>
