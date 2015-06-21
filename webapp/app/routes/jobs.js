import Ember from 'ember';

export default Ember.Route.extend({
  queryParams: {
    status: {
      refreshModel: true
    }
  },

  model: function (params) {
    if (params['status']) {
      var status = params['status'];
      return this.store.filter('job', {
        'status': status
      }, function (job) {
        return job.get('status') === status;
      });
    } else {
      return this.store.filter('job', function () {
        return true;
      });
    }
  }
});