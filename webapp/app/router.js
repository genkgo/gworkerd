import Ember from 'ember';
import config from 'gworkerd/config/environment';

var Router = Ember.Router.extend({
  location: config.locationType
});

Router.map(function() {
  this.route('login');
  this.route('jobs');
  this.route('job', {path: '/job/:job_id'});
});

export default Router;
