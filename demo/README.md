# Dasher WASM Demo

## Quickstart

1. **Build the WASM package**

   Run from the project root:
   ```sh
   wasm-pack build --target web --release
   ```
   This creates the `pkg/` directory in the project root.

2. **Serve the demo**

   Run from the project root:
   ```sh
   ./demo/build_and_serve.sh
   ```
   This will serve the `/demo` directory at http://localhost:8000

3. **Open the demo**

   Go to [http://localhost:8000/demo.html](http://localhost:8000/demo.html) in your browser.

---

## Folder Structure
- `/demo/demo.html` — Main HTML frontend
- `/demo/demo.js`   — JS frontend logic
- `/pkg/`           — WASM build output from `wasm-pack` (should be in project root!)

---

## Common Issue: 404 for `/pkg/dasher_core.js`
If you see errors like:
```
GET /pkg/dasher_core.js HTTP/1.1" 404
```
Make sure that the `pkg` directory is in the **project root**, not inside `/demo`.

- If you accidentally moved `pkg/` into `/demo`, move it back to the root.
- The import in `demo.js` should be:
  ```js
  import init, { dasher_update, dasher_reset } from '../pkg/dasher_core.js';
  ```

---

## Troubleshooting
- Always run the build and server from the project root, not `/demo`.
- If you change Rust code, rebuild with `wasm-pack build --target web --release`.
- If you still have issues, check your browser console and server logs.
