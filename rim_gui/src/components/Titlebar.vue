<script setup lang="ts">
import { invokeCommand } from "@/utils";
import { appWindow } from "@tauri-apps/api/window";
import { onMounted, Ref, ref } from "vue";

const { title } = defineProps({
    title: {
        type: String,
        default: '',
    },
});

interface Language {
    id: string
    name: string
}

const languages: Ref<Language[]> = ref([]);
// const showLangs = ref(false);

function minimize() { appWindow.minimize(); }
function maximize() { appWindow.toggleMaximize() }
function close() { appWindow.close() }
// function onLangSelected(value: string) {
//     invokeCommand("set_locale", { language: value }).then(() => {
//         location.reload();
//     });
// }

onMounted(() => {
    invokeCommand("supported_languages").then((list) => {
        if (Array.isArray(list) && list.every((item) => "id" in item && "name" in item)) {
            languages.value = list;
        }
    })
})
</script>

<template>
    <div data-tauri-drag-region class="titlebar">
        <div class="titlebar-icon" id="titlebar-icon">
            <img data-tauri-drag-region src="/favicon.ico" h="1.5rem" />
        </div>

        <div data-tauri-drag-region class="titlebar-title">{{ title }}</div>

        <div data-tauri-drag-region class="titlebar-buttons" id="titlebar-buttons">
            <!-- FIXME: we need an English translation for GUI before enabling this -->
            <!-- <div class="titlebar-button" @click="showLangs = !showLangs" @focusout="showLangs = false" tabindex="0">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
                    <path fill="white"
                        d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2m6.93 6h-2.95a15.7 15.7 0 0 0-1.38-3.56A8.03 8.03 0 0 1 18.92 8M12 4.04c.83 1.2 1.48 2.53 1.91 3.96h-3.82c.43-1.43 1.08-2.76 1.91-3.96M4.26 14C4.1 13.36 4 12.69 4 12s.1-1.36.26-2h3.38c-.08.66-.14 1.32-.14 2s.06 1.34.14 2zm.82 2h2.95c.32 1.25.78 2.45 1.38 3.56A8 8 0 0 1 5.08 16m2.95-8H5.08a8 8 0 0 1 4.33-3.56A15.7 15.7 0 0 0 8.03 8M12 19.96c-.83-1.2-1.48-2.53-1.91-3.96h3.82c-.43 1.43-1.08 2.76-1.91 3.96M14.34 14H9.66c-.09-.66-.16-1.32-.16-2s.07-1.35.16-2h4.68c.09.65.16 1.32.16 2s-.07 1.34-.16 2m.25 5.56c.6-1.11 1.06-2.31 1.38-3.56h2.95a8.03 8.03 0 0 1-4.33 3.56M16.36 14c.08-.66.14-1.32.14-2s-.06-1.34-.14-2h3.38c.16.64.26 1.31.26 2s-.1 1.36-.26 2z" />
                </svg>
                <transition name="fade" appear>
                    <div class="sub-menu" v-if="showLangs">
                        <ul v-for="item in languages" :key="item.id" class="menu-item">
                            <li @click="onLangSelected(item.id)">{{ item.name }}</li>
                        </ul>
                    </div>
                </transition>
            </div> -->

            <div class="titlebar-button" id="titlebar-minimize" @click="minimize">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 16 16">
                    <path fill="white" d="M3 8a.75.75 0 0 1 .75-.75h8.5a.75.75 0 0 1 0 1.5h-8.5A.75.75 0 0 1 3 8" />
                </svg>
            </div>

            <div class="titlebar-button" id="titlebar-maximize" @click="maximize">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 16 16">
                    <path fill="white"
                        d="M4.5 3A1.5 1.5 0 0 0 3 4.5v7A1.5 1.5 0 0 0 4.5 13h7a1.5 1.5 0 0 0 1.5-1.5v-7A1.5 1.5 0 0 0 11.5 3zM5 4.5h6a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-.5.5H5a.5.5 0 0 1-.5-.5V5a.5.5 0 0 1 .5-.5" />
                </svg>
            </div>

            <div class="titlebar-button" id="titlebar-close" @click="close">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 16 16">
                    <path fill="white" fill-rule="evenodd"
                        d="M4.28 3.22a.75.75 0 0 0-1.06 1.06L6.94 8l-3.72 3.72a.75.75 0 1 0 1.06 1.06L8 9.06l3.72 3.72a.75.75 0 1 0 1.06-1.06L9.06 8l3.72-3.72a.75.75 0 0 0-1.06-1.06L8 6.94z"
                        clip-rule="evenodd" />
                </svg>
            </div>
        </div>
    </div>
</template>

<style scoped>
.titlebar {
    background-color: rgba(2, 2, 10, 0.8);
    height: 40px;
    user-select: none;
    display: flex;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 1000;
    padding-inline: 2px;
}

.titlebar-icon {
    display: flex;
    align-items: center;
    margin-inline: 3px;
}

.titlebar-buttons {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    margin-left: auto;
    margin-right: 0;
}

.titlebar-button {
    display: flex;
    justify-content: center;
    align-items: center;
    fill: white;
    width: 32px;
    height: 32px;
    border-radius: 3px;
    margin-inline: 3px;
    padding: 0;
}

.titlebar-button:hover {
    background: #696969;
}

#titlebar-close:hover {
    background-color: #ff1528;
}

.titlebar-title {
    color: white;
    display: flex;
    align-items: center;
    margin-left: 3px;
    font-size: 12px;
}

.sub-menu {
    position: absolute;
    background-color: rgba(2, 2, 10, 0.8);
    transform: translateY(70%);
    border-radius: 3px;
}

.sub-menu ul {
    margin: 0;
    padding: 0;
}

.sub-menu ul li {
    list-style: none;
    display: flex;
    padding: 1rem;
    color: white;
    font-size: 14px;
    text-decoration: none;
}

.sub-menu ul li:hover {
  background-color: #526ecc;
}

.fade-leave-active {
    transition: all .5s ease-out;
}

.fade-leave-to {
    opacity: 0;
}
</style>
