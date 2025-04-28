import init, { dasher_get_options, dasher_accept, dasher_reset, dasher_get_context, dasher_train } from '../pkg/dasher_core.js';

let wasmReady = false;
let output = '';
let options = [];
let focusIdx = 0;
let trainingStatus = 'Not started';

function drawDemo() {
  const canvas = document.getElementById('dasher-canvas');
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Draw prediction boxes
  const stripX = canvas.width - 80;
  const stripW = 70;
  let y = 0;
  for (let i = 0; i < options.length; ++i) {
    const boxH = Math.max(options[i].prob * canvas.height, 24);
    ctx.fillStyle = (i === focusIdx) ? '#4af' : '#444';
    ctx.fillRect(stripX, y, stripW, boxH);
    ctx.fillStyle = '#fff';
    ctx.font = 'bold 24px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(options[i].symbol, stripX + stripW/2, y + boxH/2);
    y += boxH;
  }
}

function updateFromCursor(y) {
  const canvas = document.getElementById('dasher-canvas');
  const frac = Math.min(1, Math.max(0, y / canvas.height));
  // Focus index by cumulative box heights
  let total = 0;
  let found = 0;
  for (let i = 0; i < options.length; ++i) {
    const boxH = Math.max(options[i].prob * canvas.height, 24);
    if (frac * canvas.height >= total && frac * canvas.height < total + boxH) {
      found = i;
      break;
    }
    total += boxH;
  }
  focusIdx = found;
  drawDemo();
  document.getElementById('typed').textContent = output + options[focusIdx].symbol;
}

function acceptSymbol() {
  if (options.length === 0) return;
  dasher_accept(options[focusIdx].symbol);
  output = dasher_get_context();
  options = dasher_get_options();
  focusIdx = 0;
  drawDemo();
  document.getElementById('typed').textContent = output;
}

let lastY = 0;

window.addEventListener('DOMContentLoaded', async () => {
  await init();
  wasmReady = true;

  // Train the language model with sample text
  trainingStatus = 'Training...';
  document.getElementById('training-status').textContent = trainingStatus;

  // Sample training text - common English words and phrases
  const trainingText = `the quick brown fox jumps over the lazy dog.
  hello world how are you today?
  this is a sample training text for dasher.
  common words include the and of to in is that it was for on are as with his they at be this from have or by one had not but what all were when we there can an your which their said if will each about how up out them then she many some so these would other into has more her two like him see time could no make than first been its who now people my made over did down only way find use may water long little very after words called just where most know`;

  // Train the model
  const success = dasher_train(trainingText);

  // Update status
  trainingStatus = success ? 'Training complete' : 'Training failed';
  document.getElementById('training-status').textContent = trainingStatus;

  // Get options after training
  options = dasher_get_options();
  console.log('dasher_get_options returned:', options);
  if (options && typeof options === 'object' && !Array.isArray(options)) {
    // Try to convert to array if it's a plain object
    options = Object.values(options);
  }
  drawDemo();

  const canvas = document.getElementById('dasher-canvas');

  canvas.addEventListener('mousemove', e => {
    if (!wasmReady) return;
    const rect = canvas.getBoundingClientRect();
    lastY = e.clientY - rect.top;
    updateFromCursor(lastY);
  });

  canvas.addEventListener('click', e => {
    if (!wasmReady) return;
    acceptSymbol();
  });

  document.body.addEventListener('keydown', e => {
    if (e.key === 'Backspace') {
      output = output.slice(0, -1);
      document.getElementById('typed').textContent = output;
      dasher_reset();
      options = dasher_get_options();
      focusIdx = 0;
      drawDemo();
    }
    if (e.key === ' ') {
      output += ' ';
      document.getElementById('typed').textContent = output;
      dasher_reset();
      options = dasher_get_options();
      focusIdx = 0;
      drawDemo();
    }
  });
});
