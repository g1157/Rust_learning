// WebSocket 客户端 - 纯 JavaScript 实现
// 通过全局函数暴露给 WASM

(function() {
    'use strict';
    
    let ws = null;
    let messageQueue = [];
    let connectionState = 'disconnected';
    let errorMessage = '';
    
    // 辅助函数：从 WASM 内存读取字符串
    function readString(ptr, len) {
        const bytes = new Uint8Array(memory.buffer, ptr, len);
        return new TextDecoder().decode(bytes);
    }
    
    // 辅助函数：写字符串到 WASM 内存
    function writeString(str, ptr, maxLen) {
        const bytes = new TextEncoder().encode(str);
        const len = Math.min(bytes.length, maxLen);
        const mem = new Uint8Array(memory.buffer, ptr, len);
        mem.set(bytes.slice(0, len));
        return len;
    }
    
    // 全局 WebSocket API (供 Rust FFI 调用)
    window.ws_connect = function(urlPtr, urlLen) {
        const url = readString(urlPtr, urlLen);
        console.log('[WS] Connecting to:', url);
        
        connectionState = 'connecting';
        errorMessage = '';
        
        try {
            ws = new WebSocket(url);
            
            ws.onopen = function() {
                console.log('[WS] Connected!');
                connectionState = 'connected';
            };
            
            ws.onmessage = function(event) {
                console.log('[WS] Message received:', event.data);
                messageQueue.push(event.data);
            };
            
            ws.onerror = function(error) {
                console.error('[WS] Error:', error);
                connectionState = 'error';
                errorMessage = 'Connection failed';
            };
            
            ws.onclose = function() {
                console.log('[WS] Connection closed');
                connectionState = 'disconnected';
                ws = null;
            };
            
            return 1; // success
        } catch (e) {
            console.error('[WS] Connection error:', e);
            connectionState = 'error';
            errorMessage = e.message;
            return 0; // failure
        }
    };
    
    window.ws_send = function(msgPtr, msgLen) {
        const message = readString(msgPtr, msgLen);
        
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(message);
            console.log('[WS] Sent:', message);
            return 1; // success
        } else {
            console.warn('[WS] Cannot send, not connected');
            return 0; // failure
        }
    };
    
    window.ws_receive = function(bufPtr, bufLen) {
        if (messageQueue.length > 0) {
            const message = messageQueue.shift();
            return writeString(message, bufPtr, bufLen);
        }
        return 0; // no messages
    };
    
    window.ws_get_state = function(bufPtr, bufLen) {
        return writeString(connectionState, bufPtr, bufLen);
    };
    
    window.ws_is_connected = function() {
        return (ws !== null && ws.readyState === WebSocket.OPEN) ? 1 : 0;
    };
    
    window.ws_close = function() {
        if (ws) {
            ws.close();
            ws = null;
        }
        connectionState = 'disconnected';
        messageQueue = [];
    };
    
    window.ws_message_count = function() {
        return messageQueue.length;
    };
    
    console.log('[WS] WebSocket client initialized (FFI mode)');
})();
