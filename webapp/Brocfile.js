/* global require, module */

var EmberApp = require('ember-cli/lib/broccoli/ember-app');

var app = new EmberApp();

// moment
app.import('bower_components/moment/moment.js');

module.exports = app.toTree();
