<template>
    <div class="memoWrapper">
        <h1 class="memoTitle" @click="visible = !visible">
            <el-icon><Discount /></el-icon>备忘录
            <p>
                <el-tag
                    v-for="name in groupNames"
                    :key="name"
                    :type="activeGroup === name ? '' : 'info'"
                    class="groupTag"
                    @click.stop="activeGroup = name"
                >
                    {{ name }} ({{ getGroupLength(name) }})
                </el-tag>
            </p>
        </h1>
        <template v-if="visible">
            <div class="memoInput">
                <input
                    v-model="newMemo"
                    placeholder="添加备忘录"
                    @keyup.enter="addMemo"
                />
                <el-button type="primary" @click="addMemo">添加</el-button>
            </div>
            <div class="memoTags">
                <div class="memoGroup">
                    <el-tag
                        v-for="(memo, index) in displayMemos"
                        :key="memo.id"
                        class="memoTag"
                        draggable="true"
                        @dragstart="onDragStart(index)"
                        @dragover.prevent="onDragOver(index)"
                        @drop="onDrop"
                        closable
                        @close="deleteMemo(index)"
                        @click="copyMemo(memo.text)"
                    >
                        <span class="dragHandle">⋮</span>
                        <span class="memoText">{{ memo.text }}</span>
                    </el-tag>
                </div>
            </div>
        </template>
    </div>
</template>

<script setup lang="ts">
import { Discount } from '@element-plus/icons-vue'
import { ref, computed, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import type { LiveInfo } from '@/components/LiveVideo.vue'

interface Memo {
    id: number
    text: string
    liveinfoName: string
}

const props = defineProps<{
    liveInfo: LiveInfo
}>()

const MEMO_KEY = 'memo_list'

const visible = ref(true)
const memos = ref<Memo[]>([])

const newMemo = ref('')
let dragIndex = ref(-1)

const activeGroup = ref('')

const groupNames = computed(() => Object.keys(groupedMemos.value))

const groupedMemos = computed(() => {
    const groups: Record<string, Memo[]> = {}
    memos.value.forEach(memo => {
        const name = memo.liveinfoName || '默认'
        if (!groups[name]) {
            groups[name] = []
        }
        groups[name].push(memo)
    })
    return groups
})

const displayMemos = computed(() => {
    if (!activeGroup.value) return memos.value
    return groupedMemos.value[activeGroup.value] || []
})

const getGroupLength = (name: string) => {
    return groupedMemos.value[name]?.length || 0
}

watch(() => props.liveInfo.name, (newName) => {
    if (newName && groupedMemos.value[newName]) {
        activeGroup.value = newName
    }
}, { immediate: true })

const loadMemos = () => {
    const saved = localStorage.getItem(MEMO_KEY)
    if (saved) {
        memos.value = JSON.parse(saved)
    }
}

const saveMemos = () => {
    localStorage.setItem(MEMO_KEY, JSON.stringify(memos.value))
}

watch(memos, saveMemos, { deep: true })

const addMemo = () => {
    if (newMemo.value.trim()) {
        memos.value.push({
            id: Date.now(),
            text: newMemo.value.trim(),
            liveinfoName: props.liveInfo.name
        })
        newMemo.value = ''
    }
}

const deleteMemo = (index: number) => {
    const memo = displayMemos.value[index]
    const globalIndex = memos.value.findIndex(m => m.id === memo.id)
    if (globalIndex !== -1) {
        memos.value.splice(globalIndex, 1)
    }
}

const copyMemo = (text: string) => {
    navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
}

const onDragStart = (index: number) => {
    dragIndex.value = index
}

const onDragOver = (index: number) => {
    if (dragIndex.value === -1 || dragIndex.value === index) return
    
    const draggedItem = displayMemos.value[dragIndex.value]
    const displayClone = [...displayMemos.value]
    displayClone.splice(dragIndex.value, 1)
    displayClone.splice(index, 0, draggedItem)
    
    displayClone.forEach(dm => {
        const gi = memos.value.findIndex(m => m.id === dm.id)
        if (gi !== -1) {
            const original = memos.value[gi]
            memos.value.splice(gi, 1)
            memos.value.splice(gi, 0, original)
        }
    })
    dragIndex.value = index
}

const onDrop = () => {
    dragIndex.value = -1
}

onMounted(() => {
    loadMemos()
    if (props.liveInfo.name && groupedMemos.value[props.liveInfo.name]) {
        activeGroup.value = props.liveInfo.name
    }
})
</script>

<style scoped lang="scss">
.memoWrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 0 20px;
    gap: 12px;
}

.memoTitle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: bold;

    p {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin: 0;
        padding-left: 8px;
    }
}

.groupTag {
    cursor: pointer;
}

.memoInput {
    display: flex;
    align-items: center;
    gap: 15px;

    input {
        flex: 1;
        height: 32px;
        border-radius: 10px;
        padding-left: 12px;
        font-size: 14px;
        border: none;
        outline: none;
        color: black;
        background-color: #ecf0f3;
        box-shadow: inset 2px 2px 4px #d1d9e6, inset -2px -2px 4px #d1d9e6;

        &::placeholder {
            color: #999;
        }
    }
}

.memoTags {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.memoGroup {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.memoTag {
    cursor: pointer;
    
    &:active {
        cursor: grabbing;
    }
}

.dragHandle {
    margin-right: 6px;
    color: #999;
}

.memoText {
    display: inline-flex;
    max-width: 250px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
