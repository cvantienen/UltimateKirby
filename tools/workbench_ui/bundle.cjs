const fs = require('node:fs');
const path = require('node:path');
const destination = path.join(__dirname, 'vendor');
fs.mkdirSync(destination, { recursive: true });
for (const [source, name] of [
  ['chart.js/dist/chart.umd.js', 'chart.js'],
  ['chart.js/LICENSE.md', 'chart.LICENSE.md'],
  ['lucide/dist/umd/lucide.min.js', 'lucide.js'],
  ['lucide/LICENSE', 'lucide.LICENSE'],
]) fs.copyFileSync(path.join(__dirname, 'node_modules', source), path.join(destination, name));
