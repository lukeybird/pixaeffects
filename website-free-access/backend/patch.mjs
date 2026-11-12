import fs from 'node:fs';
import assert from 'node:assert/strict';
const device='app/api/pixashop/device/route.ts';let s=fs.readFileSync(device,'utf8');
const signature='async function secondsForUser(id: string) {';
assert.equal(s.split(signature).length-1,1,'Production device handler changed; refuse patch');
s=s.replace(signature,'// Pixa is free early access. Device registration, approval, rate limits and token hashing remain intact.\nasync function secondsForUser(id: string) { if (typeof id !== \'string\' || !id) return 0; const users = await sql`SELECT id FROM site_users WHERE id=${id}`; return users[0] ? 300 : 0; }\nasync function previousPaidSecondsForUser(id: string) {');
fs.writeFileSync(device,s);
for(const path of ['app/api/pixashop/download/route.ts','app/api/pixaeffects/download/route.ts']) {
 let text=fs.readFileSync(path,'utf8');assert.equal(text.split("@/app/api/billing/route").length-1,1,'Unexpected download authorization import');
 fs.writeFileSync(path,text.replace('@/app/api/billing/route','@/app/api/pixa-access/route').replaceAll('Sign in with an active trial or subscription','Sign in to your free Pixa account'));
}
for(const path of ['app/pixashop/activate/page.tsx','app/pixaeffects/activate/page.tsx']){
 let text=fs.readFileSync(path,'utf8');fs.writeFileSync(path,text.replaceAll('Manage subscription','Your Pixa account'));
}
// Required production files must survive intact; never rebuild from stale Git main.
for (const path of ['app/builder/page.tsx','app/api/billing/route.ts','lib/pixashop.ts','auth.ts','package-lock.json'])assert.ok(fs.statSync(path).size>0,path);
assert.ok(fs.readFileSync(device,'utf8').includes('tokenHash'));
assert.ok(fs.readFileSync(device,'utf8').includes('allowAuthAttempt'));
console.log('Pixa free access patch applied. Original builder/billing/auth and device safeguards retained.');
let shop=fs.readFileSync('app/pixashop/page.tsx','utf8');assert.ok(shop.includes('@/app/api/billing/route'));
shop=shop.replace('@/app/api/billing/route','@/app/api/pixa-access/route');
shop=shop.replaceAll('Start your 7-day trial','Get free access').replaceAll('Start free trial','Get free access').replaceAll('7-day free trial','free early access').replaceAll('Start your free trial','Get free access').replaceAll('Manage billing','Your Pixa account');
fs.writeFileSync('app/pixashop/page.tsx',shop);
for(const path of ['app/pixashop/activate/Activate.tsx','app/pixaeffects/activate/Activate.tsx']){
 let text=fs.readFileSync(path,'utf8');fs.writeFileSync(path,text.replaceAll('connects that device to your subscription','connects that device to your free Pixa account'));
}
