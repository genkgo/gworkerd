var auth =
{
  password: 'test'
};

module.exports = function(app) {
  var express = require('express');
  var serverRouter = express.Router();

  serverRouter.post('/', function(req, res) {
    if (!req.body || !req.body.password) {
      res.status(401);
      res.send({
        'error': 'password is required'
      });
      return;
    }

    if (req.body.password === auth.password) {
      res.send({
        'error': 'authentication successful'
      });
    } else {
      res.status(401);
      res.send({
        'error': 'authentication failed'
      });
    }
  });

  app.use('/api/auth', serverRouter);
};