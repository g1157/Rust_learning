// web/input_handler.js

// This object will store the state of all keys we are interested in.
// It is globally accessible via `window.inputState`.
window.inputState = {
    "KeyW": false,
    "KeyS": false,
    "KeyA": false,
    "KeyD": false,
    "ArrowUp": false,
    "ArrowDown": false,
    "ArrowLeft": false,
    "KeyJ": false,
    "KeyU": false,
    "ArrowRight": false,
    "Space": false,
    "Enter": false,
    "Escape": false,
    "KeyP": false,
    "KeyM": false,
    "Backspace": false,
    "PageUp": false,
    "PageDown": false,

    // Add keys for nickname input
    "KeyQ": false, "KeyE": false, "KeyR": false, "KeyT": false, "KeyY": false,
    "KeyI": false, "KeyO": false, "KeyL": false, "KeyK": false, 
    "KeyG": false, "KeyH": false, "KeyF": false, "KeyZ": false, "KeyX": false, 
    "KeyC": false, "KeyV": false, "KeyB": false, "KeyN": false,

    // Number keys
    "Digit1": false, "Digit2": false, "Digit3": false, "Digit4": false, "Digit5": false,
    "Digit6": false, "Digit7": false, "Digit8": false, "Digit9": false, "Digit0": false,
    "Numpad1": false, "Numpad4": false,
};

// We also need to track which keys were just pressed on this frame.
// This is to simulate `is_key_pressed`.
window.justPressedKeys = new Set();
window.mouseWheelDeltaY = 0;

// Listener for keydown events
window.addEventListener('keydown', (event) => {
    if (window.inputState.hasOwnProperty(event.code)) {
        // Prevent registering a "just pressed" event if the key is already held down.
        if (!window.inputState[event.code]) {
            window.justPressedKeys.add(event.code);
        }
        window.inputState[event.code] = true;
        // event.preventDefault(); // Uncomment if you want to prevent default browser actions
    }
});

// Listener for keyup events
window.addEventListener('keyup', (event) => {
    if (window.inputState.hasOwnProperty(event.code)) {
        window.inputState[event.code] = false;
        // event.preventDefault();
    }
});

// Listener for mouse wheel events
window.addEventListener('wheel', (event) => {
    // Accumulate deltaY. The value is arbitrary, but this mirrors macroquad's behavior.
    // We only care about the vertical scroll.
    window.mouseWheelDeltaY += event.deltaY > 0 ? 1.0 : (event.deltaY < 0 ? -1.0 : 0.0);
}, { passive: true });

// This function will be called from Rust at the beginning of each frame.
// It returns the state of all keys and then clears the "just pressed" keys.
function get_input_state() {
    const state = {
        down: window.inputState,
        pressed: Array.from(window.justPressedKeys),
        mouse_wheel_y: window.mouseWheelDeltaY,
    };
    // Clear the just-pressed set and mouse wheel for the next frame
    window.justPressedKeys.clear();
    window.mouseWheelDeltaY = 0;
    return JSON.stringify(state);
}

console.log("Custom input handler initialized.");
