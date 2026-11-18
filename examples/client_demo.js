/**
 * Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
 * Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.
 *
 * PaySwitch Node.js Integration Client
 */

const http = require('http');

const BASE_URL = 'http://127.0.0.1:8080';

async function checkHealth() {
  return new Promise((resolve, reject) => {
    http.get(`${BASE_URL}/health`, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          console.log(`Node Health: ${parsed.data.status} (v${parsed.data.version})`);
          resolve(parsed);
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

checkHealth().catch(err => {
  console.log(`Node connection error: ${err.message}. Start node with 'cargo run'.`);
});
