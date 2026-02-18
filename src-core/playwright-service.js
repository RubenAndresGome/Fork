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
        } else if (action === 'chat_glm') {
            await page.goto('https://chatglm.cn');
            try {
                // Selector strategies for GLM (subject to change)
                const inputSelector = 'textarea'; // Generic fallback
                await page.waitForSelector(inputSelector, { timeout: 15000 });

                if (payload && payload.prompt) {
                    await page.fill(inputSelector, payload.prompt);
                    await page.keyboard.press('Enter');

                    // Wait for response
                    await page.waitForTimeout(5000); // Basic wait

                    // Try to capture response
                    const responses = await page.$$('.markdown-body'); // Common class
                    if (responses.length > 0) {
                        const lastResponse = await responses[responses.length - 1].innerText();
                        console.log(JSON.stringify({ status: 'response_received', content: lastResponse }));
                    } else {
                        console.log(JSON.stringify({ status: 'response_received', content: "Response extraction failed (GLM)." }));
                    }
                } else {
                    console.log(JSON.stringify({ status: 'glm_opened' }));
                }
            } catch (e) {
                console.log(JSON.stringify({ status: 'error', error: 'GLM interaction failed', details: e.message }));
            }
        } else if (action === 'chat_kimi') {
            await page.goto('https://kimi.moonshot.cn');
            try {
                // Selectors for Kimi
                const inputSelector = '[contenteditable="true"]';
                await page.waitForSelector(inputSelector, { timeout: 15000 });

                if (payload && payload.prompt) {
                    await page.click(inputSelector);
                    await page.keyboard.type(payload.prompt);
                    await page.keyboard.press('Enter');

                    await page.waitForTimeout(5000);

                    // Kimi responses
                    const responses = await page.$$('.markdown');
                    if (responses.length > 0) {
                        const lastResponse = await responses[responses.length - 1].innerText();
                        console.log(JSON.stringify({ status: 'response_received', content: lastResponse }));
                    } else {
                        console.log(JSON.stringify({ status: 'response_received', content: "Response extraction failed (Kimi)." }));
                    }
                } else {
                    console.log(JSON.stringify({ status: 'kimi_opened' }));
                }
            } catch (e) {
                console.log(JSON.stringify({ status: 'error', error: 'Kimi interaction failed', details: e.message }));
            }
        } else if (action === 'chat_deepseek') {
            await page.goto('https://chat.deepseek.com');
            try {
                const inputSelector = 'textarea';
                await page.waitForSelector(inputSelector, { timeout: 15000 });

                if (payload && payload.prompt) {
                    await page.fill(inputSelector, payload.prompt);
                    await page.keyboard.press('Enter');

                    await page.waitForTimeout(5000);

                    const responses = await page.$$('.ds-markdown'); // DeepSeek specific class guess
                    if (responses.length > 0) {
                        const lastResponse = await responses[responses.length - 1].innerText();
                        console.log(JSON.stringify({ status: 'response_received', content: lastResponse }));
                    } else {
                        // Fallback attempt
                        console.log(JSON.stringify({ status: 'response_received', content: "Response extraction failed (DeepSeek)." }));
                    }
                } else {
                    console.log(JSON.stringify({ status: 'deepseek_opened' }));
                }
            } catch (e) {
                console.log(JSON.stringify({ status: 'error', error: 'DeepSeek interaction failed', details: e.message }));
            }
        } else if (action === 'chat_chatgpt') {
            await page.goto('https://chat.openai.com');
            // Esperar a que cargue el input
            try {
                const inputSelector = '#prompt-textarea';
                await page.waitForSelector(inputSelector, { timeout: 15000 });

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
