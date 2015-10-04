// Ensure export 'Module' to window in browser.
if (typeof (window) === 'object' && !window['Em']) { window['Em'] = Module; }
