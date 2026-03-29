const https = require('https');

const options = {
  hostname: 'api.github.com',
  path: '/repos/knitli/thread/pulls/comments?pull_number=127',
  method: 'GET',
  headers: {
    'User-Agent': 'node.js'
  }
};

const req = https.request(options, (res) => {
  let data = '';
  res.on('data', (chunk) => {
    data += chunk;
  });
  res.on('end', () => {
    const comments = JSON.parse(data);
    comments.forEach(c => {
      console.log(`Comment ID: ${c.id}`);
      console.log(`Path: ${c.path}`);
      console.log(`Line: ${c.line || c.original_line}`);
      console.log(`Body: ${c.body}`);
      console.log('-------------------');
    });
  });
});

req.on('error', (e) => {
  console.error(e);
});

req.end();
