const fs = require('fs')

function twoValuesRandomGenerator(a, b, n) {
  // Ensure equal occurrences of a and b
  const half = Math.floor(n / 2);
  const choices = Array(half).fill(a).concat(Array(n - half).fill(b));

  // Shuffle the array randomly
  for (let i = choices.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [choices[i], choices[j]] = [choices[j], choices[i]];  // Swap elements
  }

  return choices;
}

function randomGenerator(possibilities, n) {
  const totalOptions = possibilities.length;

  // Calculate how many times each option should ideally appear
  const occurrences = Array(totalOptions).fill(Math.floor(n / totalOptions));
  const remainder = n % totalOptions;

  // Distribute the remainder randomly
  for (let i = 0; i < remainder; i++) {
    occurrences[i]++;
  }

  // Create the array based on occurrences
  const choices = [];
  occurrences.forEach((count, index) => {
    for (let i = 0; i < count; i++) {
      choices.push(possibilities[index]);
    }
  });

  // Shuffle the array randomly
  for (let i = choices.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [choices[i], choices[j]] = [choices[j], choices[i]];  // Swap elements
  }

  return choices;
}

const possibilities = ['3m', '32m', '50m', '111m', '120k'];
const n = 100;
let result = randomGenerator(possibilities, n);
console.log(result);
fs.writeFileSync(
  '/Users/khanhtranquoc/rust/rocksdb/perf_test/keys.json',
  JSON.stringify(result),
  'utf-8'
)
