import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import vm from 'node:vm';
const root=fs.mkdtempSync(path.join(os.tmpdir(),'pixa-access-test-'));
const write=(p,s)=>{fs.mkdirSync(path.dirname(path.join(root,p)),{recursive:true});fs.writeFileSync(path.join(root,p),s);};
const originalDevice='import { sql } from "db";\nasync function secondsForUser(id: string) { return await stripe(id); }\nfunction authGuard() { return tokenHash && allowAuthAttempt && approvedUser; }';
write('app/api/pixashop/device/route.ts',originalDevice);
for(const id of ['pixashop','pixaeffects']){
 write(`app/api/${id}/download/route.ts`,"import { GET as billing } from '@/app/api/billing/route';\nconst releaseGate = true;");
 write(`app/${id}/activate/page.tsx`,'Manage subscription');
 write(`app/${id}/activate/Activate.tsx`,'This connects that device to your subscription.');
}
write('app/pixashop/page.tsx',"import { GET as billing } from '@/app/api/billing/route'; Start your free trial");
for(const p of ['app/builder/page.tsx','app/api/billing/route.ts','lib/pixashop.ts','auth.ts','package-lock.json'])write(p,'ORIGINAL-'+p);
const script=path.resolve('website-free-access/backend/patch.mjs');
let result=spawnSync(process.execPath,[script],{cwd:root,encoding:'utf8'});assert.equal(result.status,0,result.stderr);
const device=fs.readFileSync(path.join(root,'app/api/pixashop/device/route.ts'),'utf8');
assert.ok(device.includes('function authGuard() { return tokenHash && allowAuthAttempt && approvedUser; }'));
for(const p of ['app/builder/page.tsx','app/api/billing/route.ts','auth.ts'])assert.equal(fs.readFileSync(path.join(root,p),'utf8'),'ORIGINAL-'+p);
let helper=device.slice(device.indexOf('async function secondsForUser'),device.indexOf('async function previousPaidSecondsForUser')).replace('id: string','id');
let calls=0;let context={sql:async()=>{calls++;return []}};
let grant=vm.runInNewContext(helper+'; secondsForUser',context);assert.equal(await grant(null),0);assert.equal(calls,0);assert.equal(await grant('missing-user'),0);
context={sql:async()=>[{id:'real-user'}]};grant=vm.runInNewContext(helper+'; secondsForUser',context);assert.equal(await grant('real-user'),300);
write('app/api/pixashop/device/route.ts','unexpected-handler');result=spawnSync(process.execPath,[script],{cwd:root,encoding:'utf8'});assert.notEqual(result.status,0);
console.log('Passed: existing users free; unknown users denied; auth/device safeguards preserved; builder/billing untouched; source drift refuses patch.');
