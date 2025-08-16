import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import { displayConsoleBanner, updateConsoleStatus } from './utils/banner'

// Display the awesome ASCII art banner in console
displayConsoleBanner()

updateConsoleStatus('Vue app creation', 'Setting up application...')

const app = createApp(App)

app.use(createPinia())
app.use(router)

updateConsoleStatus('Application ready', 'Mounting to DOM...')

app.mount('#app')
