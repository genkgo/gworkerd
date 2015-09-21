import DS from 'ember-data';
import Ember from 'ember';

export default DS.RESTAdapter.extend({
  host: window.location.protocol + '//' + window.location.host,
  namespace: window.location.pathname.substring(1) + 'api',
  headers: Ember.computed(function() {
    return {
      "X-Password": window.sessionStorage.getItem('password')
    };
  }).volatile()
});