var server =
{
  ip: '127.0.0.1',
  hostname: 'genkgo.com',
  version: '0.1.0',
  startedAt: (new Date()).toISOString()
};

module.exports = function(app) {
  var express = require('express');
  var serverRouter = express.Router();

  serverRouter.get('/', function(req, res) {
    res.send(server);
  });

  app.use('/api/server', serverRouter);
};