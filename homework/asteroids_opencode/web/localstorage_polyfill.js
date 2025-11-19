// LocalStorage polyfill for Macroquad WASM
// Adds web-sys localStorage functions to the import object

(function() {
    'use strict';
    
    function register_localstorage(importObject) {
        // Get window and localStorage
        importObject.env.__wbg_window_5f4faef6c12b79ec = function() {
            return js_object(window);
        };
        
        importObject.env.__wbg_localStorage_3034501cd2b3da3f = function(ret, window_ptr) {
            try {
                const localStorage = js_unwrap(window_ptr).localStorage;
                if (localStorage) {
                    const localStorage_ptr = js_object(localStorage);
                    getArray(ret, Int32Array, 1)[0] = localStorage_ptr;
                    getArray(ret + 4, Int32Array, 1)[0] = 1; // Ok result
                } else {
                    getArray(ret, Int32Array, 1)[0] = 0;
                    getArray(ret + 4, Int32Array, 1)[0] = 0; // Err result
                }
            } catch (e) {
                getArray(ret, Int32Array, 1)[0] = 0;
                getArray(ret + 4, Int32Array, 1)[0] = 0;
            }
        };
        
        importObject.env.__wbg_getItem_89f57d6acc51a876 = function(ret, storage_ptr, key_ptr, key_len) {
            try {
                const storage = js_unwrap(storage_ptr);
                const key = UTF8ToString(key_ptr, key_len);
                const value = storage.getItem(key);
                
                if (value !== null) {
                    const value_ptr = js_object(value);
                    getArray(ret, Int32Array, 1)[0] = value_ptr;
                    getArray(ret + 4, Int32Array, 1)[0] = 1; // Ok(Some)
                } else {
                    getArray(ret, Int32Array, 1)[0] = 0;
                    getArray(ret + 4, Int32Array, 1)[0] = 1; // Ok(None)
                }
            } catch (e) {
                console.error('getItem error:', e);
                getArray(ret, Int32Array, 1)[0] = 0;
                getArray(ret + 4, Int32Array, 1)[0] = 0; // Err
            }
        };
        
        importObject.env.__wbg_setItem_64dfb54d7b20d84c = function(ret, storage_ptr, key_ptr, key_len, value_ptr, value_len) {
            try {
                const storage = js_unwrap(storage_ptr);
                const key = UTF8ToString(key_ptr, key_len);
                const value = UTF8ToString(value_ptr, value_len);
                storage.setItem(key, value);
                getArray(ret, Int32Array, 1)[0] = 1; // Ok
            } catch (e) {
                console.error('setItem error:', e);
                getArray(ret, Int32Array, 1)[0] = 0; // Err
            }
        };
        
        importObject.env.__wbg_removeItem_9b4a71f01eabf337 = function(ret, storage_ptr, key_ptr, key_len) {
            try {
                const storage = js_unwrap(storage_ptr);
                const key = UTF8ToString(key_ptr, key_len);
                storage.removeItem(key);
                getArray(ret, Int32Array, 1)[0] = 1; // Ok
            } catch (e) {
                console.error('removeItem error:', e);
                getArray(ret, Int32Array, 1)[0] = 0; // Err
            }
        };
        
        // Helper to unwrap JS objects from the registry
        function js_unwrap(ptr) {
            // This uses the same object registry as quad_net plugin
            if (typeof e !== 'undefined' && e[ptr]) {
                return e[ptr];
            }
            // Fallback: for window object
            if (ptr === 0 || ptr === -1) {
                return window;
            }
            return null;
        }
    }
    
    miniquad_add_plugin({
        register_plugin: register_localstorage,
        version: 1,
        name: 'web_sys_localstorage'
    });
})();
