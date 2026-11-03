// Opt-in real browser workflow. Chat/research steps make paid model calls.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

assert.equal(process.env.QA_PLAYWRIGHT_PAID, '1', 'Set QA_PLAYWRIGHT_PAID=1 only after authorizing model charges');
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE ? pathToFileURL(process.env.PLAYWRIGHT_MODULE).href : 'playwright');
const base = process.env.QA_BASE_URL || 'http://127.0.0.1:18000';
const output = process.env.QA_PLAYWRIGHT_OUTPUT || 'tests/artifacts/playwright-2026-11-03';
await fs.mkdir(output, { recursive: true });
const resume = process.env.QA_PLAYWRIGHT_RESUME === '1';
const previous = resume ? JSON.parse(await fs.readFile(path.join(output, 'report.json'), 'utf8')) : null;
const report = { started: new Date().toISOString(), base, steps: [], consoleErrors: [], requests: [] };
const browser = await chromium.launch({ headless: true, executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE, args: ['--no-sandbox'] });
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
const page = await context.newPage();
page.setDefaultTimeout(20000);
page.on('pageerror', error => report.consoleErrors.push(error.message));
page.on('response', response => {
  const url = new URL(response.url());
  if (url.pathname.startsWith('/api/')) report.requests.push({ method: response.request().method(), path: url.pathname, status: response.status() });
});
let worldId = previous?.worldId, conversationId = previous?.conversationId, lastMessages;
if (resume) {
  lastMessages = JSON.parse(await fs.readFile(path.join(output, process.env.QA_PLAYWRIGHT_FINISH_ONLY === '1' ? 'original-conversation.json' : 'conversation.json'), 'utf8')).messages;
  report.worldId = worldId; report.conversationId = conversationId; report.email = previous.email;
  report.previousSetup = [...(previous.previousSetup || []), ...previous.steps.filter(step => step.ok)];
}
const save = () => fs.writeFile(path.join(output, 'report.json'), JSON.stringify(report, null, 2));
async function step(name, action) {
  const start = Date.now();
  try {
    const detail = await action();
    report.steps.push({ name, ok: true, seconds: (Date.now() - start) / 1000, detail });
    console.log(`PASS ${name}`, JSON.stringify(detail || {}));
  } catch (error) {
    report.steps.push({ name, ok: false, seconds: (Date.now() - start) / 1000, error: error.message });
    console.log(`FAIL ${name}: ${error.message}`);
    throw error;
  } finally {
    await page.screenshot({ path: path.join(output, `${report.steps.length.toString().padStart(2, '0')}-${name}.png`), fullPage: true }).catch(() => {});
    await save();
  }
}
async function workbench(id, fields = {}, status = 200) {
  await page.getByRole('button', { name: 'API workbench', exact: true }).click();
  await page.getByLabel(/^Request/).selectOption(id);
  for (const [label, value] of Object.entries(fields)) {
    const locator = page.getByLabel(new RegExp(`^${label}`));
    if (await locator.evaluate(el => el.tagName === 'SELECT')) await locator.selectOption(value);
    else await locator.fill(value);
  }
  const method = ['retry', 'resolve'].includes(id) ? 'POST' : id === 'deleteConversation' ? 'DELETE' : 'GET';
  const response = page.waitForResponse(r => r.url().startsWith(base + '/api/world/') && r.request().method() === method);
  const name = {retry:'Retry research',resolve:'Resolve conflict',deleteConversation:'Delete conversation'}[id] || 'Send request';
  await page.getByRole('button', { name, exact: true }).click();
  const result = await response;
  await page.locator('.response-body').waitFor();
  assert.equal(result.status(), status, `${id}: ${await result.text()}`);
  const text = await result.text();
  let data = null;
  if (text) { try { data = JSON.parse(text); } catch { data = text; } }
  // Assert the response reaches the displayed UI, not only the network.
  if (data !== null) assert.deepEqual(JSON.parse(await page.locator('.response-body').innerText()), data);
  return data;
}
async function chat(text) {
  await page.getByRole('button', { name: 'Chat and stream', exact: true }).click();
  const saved = page.waitForResponse(r => new URL(r.url()).pathname === `/api/world/${worldId}/conversations/${conversationId}` && r.request().method() === 'GET', { timeout: 240000 });
  await page.locator('.composer textarea').fill(text);
  await page.getByRole('button', { name: 'Send', exact: true }).click();
  const response = await saved;
  assert.equal(response.status(), 200, 'History reload fails after chat');
  const data = await response.json();
  assert.ok(data.messages.length >= (lastMessages?.length || 0) + 2, 'Chat does not save its user and assistant messages');
  lastMessages = data.messages;
  await fs.writeFile(path.join(output, 'conversation.json'), JSON.stringify(data, null, 2));
  return data.messages.filter(m => m.role === 'user').at(-1).id;
}
async function waitResearch(messageId) {
  const deadline = Date.now() + 240000;
  while (Date.now() < deadline) {
    const { status } = await workbench('status', { 'Message ID': messageId });
    if (status === 'completed') return status;
    if (['failed', 'cancelled', 'not_submitted'].includes(status)) throw new Error(`Research ends as ${status}`);
    await page.waitForTimeout(1500);
  }
  throw new Error('Research does not finish within four minutes');
}

try {
  if (resume) {
    await step('resume-session', async () => {
      await page.goto(base);
      await page.getByLabel('Email', { exact: true }).fill(previous.email);
      await page.locator('input[type=password]').fill('Browser-Test-938!');
      await page.getByRole('button', { name: 'Log in', exact: true }).last().click();
      await page.getByRole('button', { name: 'Glasswatch browser test', exact: true }).click();
      await page.getByRole('button', { name: 'Mara and the lighthouse', exact: true }).click();
      await page.locator('.composer textarea').waitFor();
    });
  } else {
  await step('signup', async () => {
    await page.goto(base);
    await page.getByRole('tab', { name: 'Sign up', exact: true }).click();
    await page.getByLabel('Username', { exact: true }).fill('Browser Writer');
    report.email = `playwright-${Date.now()}@example.com`;
    await page.getByLabel('Email', { exact: true }).fill(report.email);
    await page.locator('input[type=password]').fill('Browser-Test-938!');
    await page.getByRole('button', { name: 'Create account', exact: true }).click();
    await page.getByRole('button', { name: 'New world', exact: true }).waitFor();
  });
  await step('create-world-and-conversation', async () => {
    await page.getByRole('button', { name: 'New world', exact: true }).click();
    await page.getByPlaceholder('Name', { exact: true }).fill('Glasswatch browser test');
    await page.getByPlaceholder('Description', { exact: true }).fill('Synthetic Playwright world, 2026-11-03');
    const created = page.waitForResponse(r => new URL(r.url()).pathname === '/api/world' && r.request().method() === 'POST');
    await page.getByRole('button', { name: 'Create world', exact: true }).click();
    worldId = (await (await created).json()).id;
    await page.getByPlaceholder('New conversation…', { exact: true }).fill('Mara and the lighthouse');
    const thread = page.waitForResponse(r => new URL(r.url()).pathname === `/api/world/${worldId}/conversations` && r.request().method() === 'POST');
    await page.getByRole('button', { name: 'Create conversation', exact: true }).click();
    conversationId = (await (await thread).json()).id;
    await page.locator('.composer textarea').waitFor();
    report.worldId = worldId; report.conversationId = conversationId;
    return { worldId, conversationId };
  });
  }
  if (process.env.QA_PLAYWRIGHT_FINISH_ONLY === '1') {
    report.failure = JSON.parse(await fs.readFile(path.join(output, 'workflow-report.json'), 'utf8')).failure;
    report.findings = previous.findings || [];
    process.exitCode = report.failure ? 1 : 0;
    const originalSource = lastMessages.filter(m => m.role === 'user')[0].id;
    const accepted = JSON.parse(await fs.readFile(path.join(output, 'accepted-entity.json'), 'utf8'));
    await step('inspect-errors-and-retry-rejection', async () => {
      await waitResearch(lastMessages.filter(m => m.role === 'user').at(-1).id);
      const entity = await workbench('entity', { 'Entity ID': accepted.id });
      report.entityAfterWrongAnswer = entity;
      await workbench('retry', { 'Message ID': originalSource }, 409);
      await workbench('entity', { 'Entity ID': 'invalid-id' }, 400);
      return { retryStatus: 409, invalidIdStatus: 400, savedAge: entity.properties.age };
    });
    await step('mobile-workbench', async () => {
      await page.setViewportSize({ width: 390, height: 844 });
      await page.getByRole('button', { name: 'Glasswatch browser test', exact: true }).waitFor({ state: 'visible' });
      assert.ok(await page.getByRole('button', { name: 'Send request', exact: true }).isVisible(), 'Mobile layout hides workbench actions');
    });
    await step('fresh-conversation-canon-check', async () => {
      await page.setViewportSize({ width: 1440, height: 1000 });
      await page.getByPlaceholder('New conversation…', { exact: true }).fill('Fresh canon check');
      const created = page.waitForResponse(r => new URL(r.url()).pathname === `/api/world/${worldId}/conversations` && r.request().method() === 'POST');
      await page.getByRole('button', { name: 'Create conversation', exact: true }).click();
      conversationId = (await (await created).json()).id;
      report.freshConversationId = conversationId;
      lastMessages = [];
      const id = await chat('How old is Mara Vale according to the currently accepted canon? Give her current age only.');
      const answer = lastMessages.filter(m => m.role === 'assistant').at(-1).content;
      report.freshAnswer = answer;
      const research = await waitResearch(id);
      assert.ok(answer.includes('41'), 'Fresh conversation also ignores accepted age');
      return { answer, research };
    });
  } else {
  let sourceId;
  await step('save-lore-and-research', async () => {
    sourceId = resume ? lastMessages.filter(message => message.role === 'user')[0].id : await chat('Mara Vale is the keeper of Glasswatch Lighthouse. She was born in Bellhaven and is 34 years old. Glasswatch Lighthouse stands in Bellhaven and is owned by the Tide Council. These are established facts for this world.');
    return { sourceId, research: await waitResearch(sourceId) };
  });
  let mara;
  await step('inspect-graph-and-source', async () => {
    const graph = await workbench('graph');
    assert.ok(graph.data.nodes.length >= 3, 'Research fails to save the stated subjects');
    assert.ok(graph.data.edges.length >= 1, 'Research fails to save the stated connections');
    mara = graph.data.nodes.find(entity => entity.name === 'Mara Vale');
    assert.ok(mara, 'Mara is absent from the graph');
    assert.ok(mara.source_message_ids.includes(sourceId), 'Entity loses its source message');
    const source = await workbench('message', { 'Message ID': sourceId });
    assert.equal(source.message.id, sourceId);
    assert.ok(source.message.content.includes('34 years old'), 'Source inspection loses the original statement');
    await fs.writeFile(path.join(output, 'initial-graph.json'), JSON.stringify(graph, null, 2));
    return { nodes: graph.data.nodes.length, edges: graph.data.edges.length, mara };
  });
  await step('retrieve-established-lore', async () => {
    const id = resume && lastMessages.filter(m => m.role === 'user').length >= 2 ? lastMessages.filter(m => m.role === 'user')[1].id : await chat('What do we know about Mara Vale and Glasswatch Lighthouse? Use only the established facts.');
    const userIndex = lastMessages.findIndex(m => m.id === id);
    const answer = lastMessages.slice(userIndex + 1).find(m => m.role === 'assistant').content;
    assert.ok(answer.includes('34') && answer.includes('Bellhaven'), 'Reply omits the stated age or birthplace');
    return { sourceId: id, research: await waitResearch(id) };
  });
  let conflict, acceptedEntity;
  await step('report-contradiction', async () => {
    const id = resume && lastMessages.filter(m => m.role === 'user').length >= 3 ? lastMessages.filter(m => m.role === 'user')[2].id : await chat("I found an older note saying Mara Vale is 41 years old, but my current notes say 34. I don't know which note is right. Keep the existing age for now and record the disagreement for review.");
    await waitResearch(id);
    const conflicts = await workbench('conflicts', { Resolved: 'false' });
    conflict = conflicts.data.find(item => item.entity_id === mara.id && JSON.stringify(item.new_value).includes('41'));
    assert.ok(conflict, 'The contradictory age produces no reviewable conflict');
    const entity = await workbench('entity', { 'Entity ID': mara.id });
    assert.deepEqual(entity.properties[conflict.field], conflict.existing_value, 'Unresolved contradiction silently changes canon');
    await fs.writeFile(path.join(output, 'conflict.json'), JSON.stringify(conflict, null, 2));
    return conflict;
  });
  await step('accept-conflict-and-read-back', async () => {
    const resolved = await workbench('resolve', { 'Conflict ID': conflict.id, Decision: 'accept_new' });
    assert.equal(resolved.resolved, true);
    const entity = await workbench('entity', { 'Entity ID': mara.id });
    assert.deepEqual(entity.properties[conflict.field], conflict.new_value, 'Accepted claim does not reach saved lore');
    acceptedEntity = entity;
    await fs.writeFile(path.join(output, 'accepted-entity.json'), JSON.stringify(entity, null, 2));
    if (/age 34\b/.test(entity.description)) report.findings = [{ issue: 'Accepted age updates the property but leaves age 34 in the entity description', entityId: mara.id, description: entity.description, properties: entity.properties }];
    return { decision: resolved.decision, field: conflict.field, value: entity.properties[conflict.field] };
  });
  await step('retrieve-accepted-canon', async () => {
    const id = await chat('How old is Mara Vale now, according to the accepted canon? Give her current age only.');
    const answer = lastMessages.filter(m => m.role === 'assistant').at(-1).content;
    const research = await waitResearch(id);
    const entity = await workbench('entity', { 'Entity ID': mara.id });
    await fs.writeFile(path.join(output, 'entity-after-question.json'), JSON.stringify(entity, null, 2));
    report.acceptedCanonCheck = { answer, savedAge: entity.properties[conflict.field] };
    assert.ok(answer.includes('41'), 'Reply does not use the accepted age');
    assert.deepEqual(entity, acceptedEntity, 'A pure question changes saved canon or source references');
    return { answer, research, savedAge: entity.properties[conflict.field] };
  });
  await step('fresh-conversation-preserves-canon', async () => {
    await page.getByPlaceholder('New conversation…', { exact: true }).fill('Fresh canon check');
    const created = page.waitForResponse(r => new URL(r.url()).pathname === `/api/world/${worldId}/conversations` && r.request().method() === 'POST');
    await page.getByRole('button', { name: 'Create conversation', exact: true }).click();
    conversationId = (await (await created).json()).id;
    report.freshConversationId = conversationId;
    lastMessages = [];
    const id = await chat('How old is Mara Vale according to currently accepted canon? Give her current age only.');
    const answer = lastMessages.filter(m => m.role === 'assistant').at(-1).content;
    const research = await waitResearch(id);
    const entity = await workbench('entity', { 'Entity ID': mara.id });
    report.freshAnswer = answer;
    assert.ok(answer.includes('41'), 'Fresh conversation ignores the accepted age');
    assert.deepEqual(entity, acceptedEntity, 'Fresh question changes saved canon or source references');
    return { answer, research, savedAge: entity.properties[conflict.field] };
  });
  await step('inspect-errors-and-retry-rejection', async () => {
    const error = await workbench('retry', { 'Message ID': sourceId }, 409);
    assert.ok(error.message || error.error, 'UI has no recovery error detail');
    await workbench('entity', { 'Entity ID': 'invalid-id' }, 400);
    return { retryStatus: 409, invalidIdStatus: 400 };
  });
  }
  assert.deepEqual(report.consoleErrors, [], 'Browser reports uncaught UI errors');
} catch (error) {
  if (report.failure) report.followupFailure = error.stack;
  else report.failure = error.stack;
  process.exitCode = 1;
} finally {
  report.finished = new Date().toISOString();
  if (report.failure && process.env.QA_PLAYWRIGHT_FINISH_ONLY !== '1') {
    await fs.writeFile(path.join(output, 'workflow-report.json'), JSON.stringify(report, null, 2));
    if (lastMessages) await fs.writeFile(path.join(output, 'original-conversation.json'), JSON.stringify({ messages: lastMessages }, null, 2));
  }
  await save();
  await browser.close();
}
