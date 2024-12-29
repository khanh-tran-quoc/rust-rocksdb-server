const fs = require('fs')

// const countFilePath = '/Users/khanhtranquoc/rust/rocksdb/perf_test/count.txt';
const keyFilePath = '/Users/khanhtranquoc/rust/rocksdb/perf_test/keys.json';
const keys = JSON.parse(fs.readFileSync(keyFilePath, 'utf-8'));
const methodFilePath = '/Users/khanhtranquoc/rust/rocksdb/perf_test/methods.json';
const methods = JSON.parse(fs.readFileSync(methodFilePath, 'utf8'))

let count = 0;
let map = {};

const beforeRequest = (requestParams, context, ee, next) => {
  // const count = parseInt(fs.readFileSync(countFilePath, 'utf-8'));
  const key = keys[count % 100];
  const method = methods[count % 100];
  count += 1;
  console.log(count);

  const valuePath = `/Users/khanhtranquoc/rust/rocksdb/perf_test/input/${key}.json`
  let value = '';
  if (valuePath in map) {
    value = map[valuePath]
  } else {
    value = fs.readFileSync(valuePath, 'utf-8')
    map[valuePath] = value
  }
  // const value = fs.readFileSync(valuePath, 'utf-8')

  requestParams.url = requestParams.url + method + `?key=${key}`;
  if (method === 'put') {
    context.vars.requestBody = value;
  }
  console.log(method, key, context.vars?.requestBody?.length);

  // fs.writeFileSync(countFilePath, (count + 1).toString(), 'utf-8')

  return next();
}

module.exports = {
  beforeRequest
};
