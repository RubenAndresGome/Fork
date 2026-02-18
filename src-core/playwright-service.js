// @ts-nocheck
const { chromium } = require('playwright');
const readline = require('readline');
const fs = require('fs');
const path = require('path');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
});

let context;
let page;
const USER_DATA_DIR = path.join(process.env.APPDATA || process.env.HOME, 'CodeChatUniversal', 'browser_data');

async function initBrowser() {
    if (!fs.existsSync(USER_DATA_DIR)) {
        fs.mkdirSync(USER_DATA_DIR, { recursive: true });
    }

    // Usamos launchPersistentContext para mantener cookies/sesión entre reinicios
    context = await chromium.launchPersistentContext(USER_DATA_DIR, {
        headless: false, // Visible para login manual inicial
        viewport: { width: 1280, height: 720 }
    });

    page = context.pages().length > 0 ? context.pages()[0] : await context.newPage();
}

async function handleMessage(message) {
    try {
        const { action, payload } = JSON.parse(message);

        if (action === 'init') {
            await initBrowser();
            console.log(JSON.stringify({ status: 'ready' }));
        } else if (action === 'navigate') {
            await page.goto(payload.url);
            console.log(JSON.stringify({ status: 'navigated', url: payload.url }));
        } else if (action === 'chat_deepseek') {
            await page.goto('https://chat.deepseek.com');
            // TODO: Implementar selectores reales
            console.log(JSON.stringify({ status: 'deepseek_opened' }));
        } else if (action === 'chat_chatgpt') {
            await page.goto('https://chat.openai.com');
            // Esperar a que cargue el input
            try {
                const inputSelector = '#prompt-textarea';
                await page.waitForSelector(inputSelector, { timeout: 10000 });

                if (payload && payload.prompt) {
                    await page.fill(inputSelector, payload.prompt);
                    await page.keyboard.press('Enter');
                    // Esperar respuesta (implementación básica)
                    // En producción usaríamos mutation observer o esperar a que el botón de stop desaparezca
                    await page.waitForTimeout(5000);

                    // Extraer último mensaje del asistente
                    const responses = await page.$$('[data-message-author-role="assistant"]');
                    if (responses.length > 0) {
                        const lastResponse = await responses[responses.length - 1].innerText();
                        console.log(JSON.stringify({ status: 'response_received', content: lastResponse }));
                    } else {
                        console.log(JSON.stringify({ status: 'response_received', content: "No se pudo extraer la respuesta." }));
                    }
                } else {
                    console.log(JSON.stringify({ status: 'chatgpt_opened' }));
                }
            } catch (e) {
                console.log(JSON.stringify({ status: 'error', error: 'Login required or timeout', details: e.message }));
            }
        } else if (action === 'close') {
            if (context) await context.close();
            process.exit(0);
        }
    } catch (error) {
        console.error(JSON.stringify({ error: error.message }));
    }
}

rl.on('line', (line) => {
    handleMessage(line);
});
