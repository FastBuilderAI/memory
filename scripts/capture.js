const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

const domains = [
    "health_science",
    "robotics",
    "driverless_cars",
    "business_analytics",
    "email_analysis",
    "audit",
    "world_events"
];

async function capture() {
    const browser = await puppeteer.launch({ headless: "new" });
    const page = await browser.newPage();
    
    // Set viewport to capture full graph clearly
    await page.setViewport({ width: 1200, height: 900 });

    for (const domain of domains) {
        console.log(`Processing ${domain}...`);
        
        const filePath = path.resolve(__dirname, `../example/${domain}/index.html`);
        const fileUrl = `file://${filePath}`;
        
        await page.goto(fileUrl, { waitUntil: 'networkidle2' });
        
        // Wait for D3 animation to settle
        await new Promise(r => setTimeout(r, 2000));
        
        // Screenshot Before State
        await page.screenshot({ path: path.resolve(__dirname, `../example/${domain}/before.png`) });
        console.log(`  Saved before.png`);
        
        // Click the After button
        await page.click('#btn-after');
        
        // Wait for D3 animation to settle again
        await new Promise(r => setTimeout(r, 2000));
        
        // Screenshot After State
        await page.screenshot({ path: path.resolve(__dirname, `../example/${domain}/after.png`) });
        console.log(`  Saved after.png`);
    }

    await browser.close();
    console.log("Screenshots completed successfully.");
}

capture().catch(console.error);
