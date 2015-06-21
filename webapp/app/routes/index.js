import Ember from 'ember';
import ENV from 'gworkerd/config/environment';

export default Ember.Route.extend({
  jobLimit : ENV.APP.initialJobLimit,

  model: function () {
    return this.store.filter('job', {
      limit: this.get('jobLimit')
    }, function () {
      return true;
    });
  }

});