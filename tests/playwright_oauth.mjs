// UI contract test. Controlled provider/session fixtures; no real Google sign-in.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
const {chromium} = await import(process.env.PLAYWRIGHT_MODULE ? pathToFileURL(process.env.PLAYWRIGHT_MODULE).href : 'playwright');
const base = process.env.QA_BASE_URL || 'http://127.0.0.1:5174';
const output = 'tests/artifacts/playwright-oauth-2026-11-10';
await fs.mkdir(output,{recursive:true});
const browser = await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE,args:['--no-sandbox']});
const report = {scope:'Playground UI with controlled OAuth/session fixtures; no live provider calls',checks:[]};
try {
  for (const scenario of ['success','cancel','blocked','session-error']) {
    const context = await browser.newContext();
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    let failSession = false;
    await context.route('**/api/**',async route => {
      const pathname = new URL(route.request().url()).pathname;
      if (pathname === '/api/auth/google') return route.fulfill({contentType:'text/html',body:'<button onclick="document.cookie=\'qa_oauth=done; path=/\'">Finish test sign-in</button>'});
      if (pathname === '/api/auth/me') {
        const authenticated = (await context.cookies()).some(c => c.name === 'qa_oauth');
        return route.fulfill({status:failSession ? 503 : authenticated ? 200 : 401,contentType:'application/json',body:authenticated ? JSON.stringify({id:'qa-user',username:'OAuth Writer',email:'qa@example.test'}) : '{}'});
      }
      return route.fulfill({contentType:'application/json',body:'[]'});
    });
    await page.goto(base);
    const button = page.getByRole('button',{name:'Continue with Google'});
    await button.waitFor();
    if (scenario === 'blocked') {
      await page.evaluate(() => {window.open = () => null;});
      await button.click();
      await page.getByRole('alert').filter({hasText:'Allow pop-ups'}).waitFor();
    } else {
      const opened = page.waitForEvent('popup');
      await button.click();
      const popup = await opened;
      await popup.waitForLoadState();
      if (scenario === 'success') {
        await popup.getByRole('button',{name:'Finish test sign-in'}).click();
        await page.getByText('OAuth Writer',{exact:true}).waitFor();
        assert.equal(popup.isClosed(),true,'Successful login must close the popup');
      } else if (scenario === 'cancel') {
        await popup.close();
        await page.getByRole('alert').filter({hasText:'did not finish'}).waitFor();
        assert.equal(await button.isEnabled(),true,'Cancellation must allow another login');
      } else {
        failSession = true;
        await page.getByRole('alert').filter({hasText:'Could not check your session'}).waitFor();
        assert.equal(popup.isClosed(),true,'Session errors must close the popup');
      }
    }
    assert.deepEqual(errors,[],'OAuth UI must not throw browser errors');
    await page.screenshot({path:`${output}/${scenario}.png`,fullPage:true});
    report.checks.push({scenario,passed:true});
    console.log(`PASS ${scenario}`);
    await context.close();
  }
} finally {
  await fs.writeFile(`${output}/report.json`,JSON.stringify(report,null,2));
  await browser.close();
}
