import init, { dasher_get_options, dasher_accept, dasher_reset, dasher_get_context, dasher_train } from '../pkg/dasher_core.js';

// Global state
let wasmReady = false;
let output = '';
let options = [];
let trainingStatus = 'Not started';
let animationFrameId = null;
let lastTimestamp = 0;
let mouseX = 0;
let mouseY = 0;
let isMouseDown = false;
let zoomBoxes = [];
let canvas = null;
let ctx = null;
let canvasWidth = 0;
let canvasHeight = 0;
let crosshair = null;
let isZooming = false;
let originX = 0; // X-coordinate of the origin (center of screen)

// Constants
const ZOOM_SPEED = 0.0001; // Speed of zooming (extremely slow for more control)
const MIN_BOX_WIDTH = 100;  // Minimum width of a box in pixels
const MIN_BOX_HEIGHT = 30;  // Minimum height of a box in pixels
const FONT_SIZE_RATIO = 0.5; // Font size as a ratio of box height
const DEBUG = true; // Enable debug information
const COLORS = [
  '#4285F4', '#EA4335', '#FBBC05', '#34A853', // Google colors
  '#3498db', '#e74c3c', '#2ecc71', '#f39c12', // Flat UI colors
  '#9b59b6', '#1abc9c', '#d35400', '#c0392b',
  '#16a085', '#8e44ad', '#27ae60', '#2980b9'
];

// Initialize the demo
window.addEventListener('DOMContentLoaded', async () => {
  // Initialize WebAssembly module
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

  // Set up the canvas
  setupCanvas();

  // Get initial options
  updateOptions();

  // Create initial zoom boxes
  createZoomBoxes();

  // Start with zooming paused - user needs to click to start
  isZooming = false;

  // Start the animation loop
  startAnimation();

  // Set up event listeners
  setupEventListeners();
});

function setupCanvas() {
  canvas = document.getElementById('dasher-canvas');
  ctx = canvas.getContext('2d');
  crosshair = document.getElementById('crosshair');

  // Set canvas size to match window
  resizeCanvas();
  window.addEventListener('resize', resizeCanvas);
}

function resizeCanvas() {
  canvasWidth = window.innerWidth;
  canvasHeight = window.innerHeight;
  canvas.width = canvasWidth;
  canvas.height = canvasHeight;

  // Set the origin to the center of the screen horizontally
  originX = canvasWidth / 2;
}

function setupEventListeners() {
  // Mouse movement
  canvas.addEventListener('mousemove', (e) => {
    if (!wasmReady) return;

    mouseX = e.clientX;
    mouseY = e.clientY;

    // Update crosshair position
    crosshair.style.left = `${mouseX}px`;
    crosshair.style.top = `${mouseY}px`;

    // Start zooming when mouse moves
    isZooming = true;
  });

  // Mouse down/up for pausing/resuming
  canvas.addEventListener('mousedown', () => {
    isMouseDown = true;
    isZooming = !isZooming; // Toggle zooming on click
  });

  canvas.addEventListener('mouseup', () => {
    isMouseDown = false;
  });

  // Touch events for mobile
  canvas.addEventListener('touchmove', (e) => {
    if (!wasmReady) return;
    e.preventDefault();

    const touch = e.touches[0];
    mouseX = touch.clientX;
    mouseY = touch.clientY;

    // Update crosshair position
    crosshair.style.left = `${mouseX}px`;
    crosshair.style.top = `${mouseY}px`;

    // Start zooming when touch moves
    isZooming = true;
  });

  canvas.addEventListener('touchend', (e) => {
    e.preventDefault();
    isZooming = !isZooming; // Toggle zooming on touch end
  });

  // Keyboard events
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Backspace') {
      output = output.slice(0, -1);
      document.getElementById('typed').textContent = output;
      dasher_reset();
      updateOptions();
      createZoomBoxes();
    }
    if (e.key === ' ') {
      output += ' ';
      document.getElementById('typed').textContent = output;
      dasher_reset();
      updateOptions();
      createZoomBoxes();
    }
    if (e.key === 'Escape') {
      isZooming = !isZooming; // Toggle zooming with Escape key
    }
  });
}

function updateOptions() {
  options = dasher_get_options();
  console.log('dasher_get_options returned:', options);
  if (options && typeof options === 'object' && !Array.isArray(options)) {
    // Try to convert to array if it's a plain object
    options = Object.values(options);
  }
}

function createZoomBoxes() {
  // Clear existing zoom boxes
  zoomBoxes = [];

  // Create new zoom boxes based on options
  if (!options || options.length === 0) {
    console.warn("No options available for creating zoom boxes");
    return;
  }

  console.log("Creating zoom boxes with", options.length, "options");

  // Calculate total probability
  const totalProb = options.reduce((sum, option) => sum + option.prob, 0);

  // Initial box width - starts at 1/3 of the screen width
  const initialBoxWidth = canvasWidth / 3;

  // Calculate cumulative heights
  let cumulativeTop = 0;

  options.forEach((option, index) => {
    // Calculate height proportion based on probability
    const heightProportion = option.prob / totalProb;
    const height = canvasHeight * heightProportion;

    // Store box data - all boxes start at the right edge
    zoomBoxes.push({
      symbol: option.symbol,
      prob: option.prob,
      x: canvasWidth - initialBoxWidth, // Start at the right edge of the screen
      y: cumulativeTop,
      width: initialBoxWidth,
      height: height,
      color: COLORS[index % COLORS.length],
      index: index
    });

    cumulativeTop += height;
  });

  console.log("Created", zoomBoxes.length, "zoom boxes");

  // Draw the initial state
  drawZoomBoxes();
}

function startAnimation() {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }

  lastTimestamp = performance.now();
  animationFrameId = requestAnimationFrame(animate);
}

function animate(timestamp) {
  const deltaTime = timestamp - lastTimestamp;
  lastTimestamp = timestamp;

  // Update zoom boxes based on mouse position
  if (isZooming && wasmReady) {
    updateZoom(deltaTime);
  }

  // Draw the current state
  drawZoomBoxes();

  // Check for selections (boxes crossing the origin)
  checkForSelections();

  // Update debug information
  if (DEBUG) {
    updateDebugInfo();
  }

  // Continue animation
  animationFrameId = requestAnimationFrame(animate);
}

function updateDebugInfo() {
  const debugElement = document.getElementById('debug-info');
  const mouseRelativeToOrigin = mouseX - originX;
  const direction = Math.sign(mouseRelativeToOrigin);
  const directionText = direction > 0 ? 'Forward' : (direction < 0 ? 'Backward' : 'Stopped');

  // Count visible boxes
  const visibleBoxes = zoomBoxes.filter(box =>
    box.x + box.width > 0 && box.x < canvasWidth
  ).length;

  // Get first box info
  let firstBoxInfo = 'No boxes';
  if (zoomBoxes.length > 0) {
    const box = zoomBoxes[0];
    firstBoxInfo = `Box 0: x=${Math.round(box.x)}, w=${Math.round(box.width)}, sym=${box.symbol}`;
  }

  debugElement.textContent = `Direction: ${directionText}, Visible: ${visibleBoxes}, ${firstBoxInfo}`;
}

function drawZoomBoxes() {
  // Clear the canvas
  ctx.clearRect(0, 0, canvasWidth, canvasHeight);

  // Draw each zoom box
  if (zoomBoxes.length > 0) {
    zoomBoxes.forEach(box => {
      // Only draw boxes that are at least partially visible
      if (box.x + box.width > 0 && box.x < canvasWidth) {
        // Draw the box
        ctx.fillStyle = box.color;
        ctx.fillRect(box.x, box.y, box.width, box.height);

        // Draw the symbol
        ctx.fillStyle = 'white';
        ctx.font = `${Math.max(box.height * FONT_SIZE_RATIO, 24)}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(box.symbol, box.x + box.width / 2, box.y + box.height / 2);

        // Draw border
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
        ctx.strokeRect(box.x, box.y, box.width, box.height);
      }
    });
  } else {
    // If no boxes, display a message
    ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
    ctx.font = '24px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('No options available - click to restart', canvasWidth / 2, canvasHeight / 2);
  }

  // Draw a vertical line at the origin
  ctx.beginPath();
  ctx.moveTo(originX, 0);
  ctx.lineTo(originX, canvasHeight);
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
  ctx.lineWidth = 2;
  ctx.stroke();

  // Add a label for the origin line
  ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
  ctx.font = '14px sans-serif';
  ctx.textAlign = 'center';
  ctx.fillText('Selection Line', originX, 20);
}

function updateZoom(deltaTime) {
  if (zoomBoxes.length === 0) return;

  // Calculate which box the mouse is in
  const mouseYRatio = mouseY / canvasHeight;
  let targetBoxIndex = -1;
  let cumulativeHeight = 0;

  for (let i = 0; i < zoomBoxes.length; i++) {
    const box = zoomBoxes[i];
    cumulativeHeight += box.height / canvasHeight;

    if (mouseYRatio <= cumulativeHeight) {
      targetBoxIndex = i;
      break;
    }
  }

  if (targetBoxIndex === -1) return;

  // Calculate zoom effect - in Dasher, horizontal mouse position controls direction and speed
  // Mouse to the left of origin = backward movement (right to left)
  // Mouse to the right of origin = forward movement (left to right)
  const mouseRelativeToOrigin = mouseX - originX;
  const direction = Math.sign(mouseRelativeToOrigin); // -1 for left of origin, +1 for right of origin
  const speedFactor = Math.abs(mouseRelativeToOrigin) / (canvasWidth / 2); // 0 at origin, 1 at edges
  const zoomAmount = direction * ZOOM_SPEED * deltaTime * (0.1 + speedFactor * 2); // Direction and speed based on cursor position

  // Update each box
  let newTotalHeight = 0;
  zoomBoxes.forEach((box, index) => {
    // Boxes closer to the target grow, others shrink
    let heightChange = 1.0;

    if (index === targetBoxIndex) {
      // Target box grows faster
      heightChange = 1.0 + zoomAmount * 2;
    } else {
      // Other boxes shrink
      const distance = Math.abs(index - targetBoxIndex);
      heightChange = 1.0 - (zoomAmount / (distance + 1));
    }

    // Apply height change
    box.height *= heightChange;

    // Ensure minimum height
    box.height = Math.max(box.height, MIN_BOX_HEIGHT);

    // Track total height for normalization
    newTotalHeight += box.height;

    // Move boxes based on direction (positive zoomAmount = forward/left, negative = backward/right)
    // The closer to the target, the faster they move
    const moveSpeedFactor = Math.abs(zoomAmount) * (1.0 + (index === targetBoxIndex ? 1.0 : 0));
    box.x -= box.width * moveSpeedFactor * Math.sign(zoomAmount); // Move left or right based on direction
  });

  // Normalize heights to fit canvas
  const scaleFactor = canvasHeight / newTotalHeight;
  let cumulativeTop = 0;

  zoomBoxes.forEach(box => {
    // Scale height
    box.height *= scaleFactor;

    // Update vertical position
    box.y = cumulativeTop;
    cumulativeTop += box.height;
  });
}

function checkForSelections() {
  // Only check for selections if we're zooming and we have boxes
  if (!isZooming || zoomBoxes.length === 0) return;

  // Get the direction of movement based on mouse position
  const mouseRelativeToOrigin = mouseX - originX;
  const direction = Math.sign(mouseRelativeToOrigin); // -1 for left of origin, +1 for right of origin

  // Don't select if mouse is too close to origin (prevents accidental selections)
  if (Math.abs(mouseRelativeToOrigin) < 20) return;

  // Check if any boxes have crossed the origin
  for (let i = 0; i < zoomBoxes.length; i++) {
    const box = zoomBoxes[i];
    const boxCenter = box.x + (box.width / 2);

    // For forward movement (direction > 0), select when box center crosses origin from right to left
    // For backward movement (direction < 0), select when box center crosses origin from left to right
    if ((direction > 0 && boxCenter <= originX && boxCenter > originX - 10 && box.x + box.width > 0) ||
        (direction < 0 && boxCenter >= originX && boxCenter < originX + 10 && box.x < canvasWidth)) {

      console.log("Selecting character:", box.symbol, "at position:", boxCenter);

      // Select this character
      selectCharacter(box.symbol);
      break;
    }
  }
}

function selectCharacter(symbol) {
  // Accept the symbol
  dasher_accept(symbol);

  // Update output
  output = dasher_get_context();
  document.getElementById('typed').textContent = output;

  // Get new options
  updateOptions();

  // Create new zoom boxes
  createZoomBoxes();

  // If we have no options, reset to get the initial options
  if (zoomBoxes.length === 0) {
    console.log("No options available after selection, resetting");
    resetDasher();
  }
}

function resetDasher() {
  // Reset the Dasher context
  dasher_reset();

  // Get initial options
  updateOptions();

  // Create new zoom boxes
  createZoomBoxes();
}

function handleClick() {
  // If we have no options, reset
  if (zoomBoxes.length === 0) {
    resetDasher();
    isZooming = true;
    return;
  }

  // Toggle zooming on click
  isZooming = !isZooming;
}
