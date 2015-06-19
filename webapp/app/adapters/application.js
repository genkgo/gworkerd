import DS from 'ember-data';
import ENV from 'gworkerd/config/environment';

export default DS.RESTAdapter.extend({
  host: ENV.APP.data.host,
  namespace: ENV.APP.data.path
});