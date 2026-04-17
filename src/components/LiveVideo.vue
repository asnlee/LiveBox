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
        <div id="dplayer" class="dplayer"></div>
        <div v-if="liveInfo.status === 4" class="over">直播已结束</div>
    </div>
</template>

<script setup lang="ts">
import DPlayer from 'dplayer'
import Hls from 'hls.js'
import Flv from 'flv.js'

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

const captureScreenshot = (video: HTMLVideoElement) => {
    if (!video) return;

    const canvas = document.createElement('canvas');
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext('2d') as any;
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height);

    const link = document.createElement('a');
    const time = new Date().toLocaleString().split(' ').pop() || Date.now();
    link.download = `${props.liveInfo.name}(${time}).png`;
    link.href = canvas.toDataURL('image/png');
    link.click();
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
