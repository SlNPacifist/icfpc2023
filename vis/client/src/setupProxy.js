const { createProxyMiddleware } = require('http-proxy-middleware');
module.exports = function(app) {
    app.use(createProxyMiddleware('/api/', // replace with your endpoint
        { target: 'http://localhost:8000' } // replace with your target
    ));
}