/* jshint node: true */

module.exports = function(environment) {
  var ENV = {
    modulePrefix: 'gworkerd',
    environment: environment,
    baseURL: '/',
    rootURL: 'index.html',
    locationType: 'hash',
    EmberENV: {
      FEATURES: {
        // Here you can enable experimental features on an ember canary build
        // e.g. 'with-controller': true
      }
    },

    APP: {
      initialJobLimit: 300,
      data : {
        host: '',
        path: 'api',
        socket: 'ws://localhost:3000/stream'
      }
    }
  };

  ENV.contentSecurityPolicy = {
    'script-src': "'self' 'unsafe-eval'",
    'img-src': "'self' https://static.genkgo.com",
    'font-src': "'self' https://fonts.googleapis.com https://fonts.gstatic.com",
    'style-src': "'self' https://fonts.googleapis.com",
    'connect-src': "'self' ws://localhost:4200 localhost:4200"
  };

  if (environment === 'development') {
    // ENV.APP.LOG_RESOLVER = true;
    // ENV.APP.LOG_ACTIVE_GENERATION = true;
    // ENV.APP.LOG_TRANSITIONS = true;
    // ENV.APP.LOG_TRANSITIONS_INTERNAL = true;
    // ENV.APP.LOG_VIEW_LOOKUPS = true;
    ENV.APP.data = {
      host: 'http://localhost:4200',
      path: 'api',
      socket: 'ws://localhost:4200/api/stream'
    };
  }

  if (environment === 'staging') {
    ENV.APP.initialJobLimit = 1;
  }

  if (environment === 'test') {
    // Testem prefers this...
    ENV.baseURL = '/';
    ENV.locationType = 'none';

    // keep test console output quieter
    ENV.APP.LOG_ACTIVE_GENERATION = false;
    ENV.APP.LOG_VIEW_LOOKUPS = false;
    ENV.APP.rootElement = '#ember-testing';
  }

  if (environment === 'production') {
    ENV.baseURL = '/monitor';
  }

  return ENV;
};
