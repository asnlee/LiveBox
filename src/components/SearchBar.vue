<template>
    <div class="inputWrapper" @click.stop>
        <input
            class="urlInput"
            v-model="inputUrl"
            placeholder="请输入直播间地址"
            @focus="showHistory = true"
        />
        <div v-if="showHistory && historyList.length" class="historyDropdown">
            <div
                v-for="(item, index) in historyList"
                :key="index"
                class="historyItem"
                @click.stop="selectHistory(item)"
            >
                <div class="historyInfo">
                    <div class="historyTitle">{{ item.title }}</div>
                    <div class="historyMeta">
                        {{ item.anchor_name }} · {{ item.roomId }}
                    </div>
                </div>
                <el-icon class="deleteIcon" @click.stop="deleteHistory(index)">
                    <Close />
                </el-icon>
            </div>
        </div>
    </div>

    <el-button type="primary" class="startListen" :loading="loading" :disabled="loading" @click="handleStart">
        开始采集
    </el-button>

    <!-- <el-button type="primary" class="startListen" @click="handleOpenWindow">
        新窗口
    </el-button> -->
</template>

<script setup lang="ts">
import { Close } from '@element-plus/icons-vue'
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{
    (e: 'start', url: string): void
    (e: 'openWindow', url: string): void
}>()

const inputUrl = ref(localStorage.getItem('url') || '')
const showHistory = ref(false)
const historyList = ref<{ title: string; anchor_name: string; roomId: string; url: string }[]>([])

const HISTORY_KEY = 'url_history'
const MAX_HISTORY = 10

const loadHistory = () => {
    const saved = localStorage.getItem(HISTORY_KEY)
    if (saved) {
        historyList.value = JSON.parse(saved)
    }
}

const saveToHistory = (roomInfo: { title: string; anchor_name: string; roomId: string; url: string }) => {
    const filtered = historyList.value.filter((item) => item.roomId !== roomInfo.roomId)
    filtered.unshift(roomInfo)
    historyList.value = filtered.slice(0, MAX_HISTORY)
    localStorage.setItem(HISTORY_KEY, JSON.stringify(historyList.value))
}

const selectHistory = (item: { title: string; anchor_name: string; roomId: string; url: string }) => {
    inputUrl.value = item.url
    showHistory.value = false
    handleStart()
}

const deleteHistory = (index: number) => {
    historyList.value.splice(index, 1)
    localStorage.setItem(HISTORY_KEY, JSON.stringify(historyList.value))
}

const closeHistory = (e: MouseEvent) => {
    const target = e.target as HTMLElement
    if (!target.closest('.inputWrapper')) {
        showHistory.value = false
    }
}

let timer: NodeJS.Timeout | null = null
const loading = ref(false)
const handleStart = () => {
    if (loading.value) { return }

    loading.value = true
    localStorage.setItem('url', inputUrl.value)
    emit('start', inputUrl.value)
    timer = setTimeout(() => {
        loading.value = false
    }, 1500)
}

const handleOpenWindow = () => {
    emit('openWindow', inputUrl.value)
}

defineExpose({
    saveToHistory
})

onMounted(() => {
    loadHistory()
    document.addEventListener('click', closeHistory)
})

onUnmounted(() => {
    document.removeEventListener('click', closeHistory)
})
</script>

<style scoped lang="scss">
.inputWrapper {
    position: relative;
    width: 50%;

    .urlInput {
        width: 100%;
        height: 36px;
        border-radius: 10px;
        padding-left: 16px;
        font-size: 15px;
        letter-spacing: 0.15px;
        border: none;
        outline: none;
        color: black;
        font-family: 'Montserrat', sans-serif;
        background-color: #ecf0f3;
        transition: 0.25s ease;
        box-shadow: inset 2px 2px 4px #d1d9e6, inset -2px -2px 4px #d1d9e6;

        &:focus {
            box-shadow: inset 4px 4px 4px #d1d9e6,
                inset -4px -4px 4px #e1e5ec;
        }
    }

    .historyDropdown {
        position: absolute;
        top: 40px;
        left: 0;
        right: 0;
        background: #ecf0f3;
        border-radius: 10px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        z-index: 1000;
        max-height: 320px;
        overflow-y: auto;

        .historyItem {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px 16px;
            cursor: pointer;
            border-radius: 10px;
            color: #333;
            font-size: 14px;

            &:hover {
                background: #d1d9e6;
            }

            .historyInfo {
                flex: 1;
                overflow: hidden;

                .historyTitle {
                    font-weight: 500;
                    white-space: nowrap;
                    overflow: hidden;
                    text-overflow: ellipsis;
                }

                .historyMeta {
                    font-size: 12px;
                    color: #666;
                    margin-top: 2px;
                }
            }

            .deleteIcon {
                margin-left: 10px;
                opacity: 0;
                transition: opacity 0.2s;

                &:hover {
                    color: #f56c6c;
                }
            }

            &:hover .deleteIcon {
                opacity: 1;
            }
        }
    }
}

.startListen {
    margin-left: 16px;
    box-shadow: 0 0 6px 2px #bfc7d4;
}
</style>
