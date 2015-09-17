import DS from 'ember-data';
import ENV from 'gworkerd/config/environment';

export default DS.RESTAdapter.extend({
  host: ENV.APP.data.host || window.location.protocol + '//' + window.location.host,
  namespace: ENV.APP.data.path
});