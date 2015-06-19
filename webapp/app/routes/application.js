import Ember from 'ember';
import fetch from 'fetch';

export default Ember.Route.extend({
  model: function () {
    return fetch('/api/server').then(function(request) {
      return request.json();
    });
  }
});