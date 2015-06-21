import Ember from 'ember';
import fetch from 'fetch';

export default Ember.Route.extend({
  model: function () {
    return fetch('/api/server').then(function(request) {
      return request.json();
    });
  },

  actions : {
    jobUpdate : function (job) {
      this.store.push('job', this.store.normalize('job', job));
    }
  }
});