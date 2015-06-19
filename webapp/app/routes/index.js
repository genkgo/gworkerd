import Ember from 'ember';
import ENV from 'gworkerd/config/environment';

export default Ember.Route.extend({
  jobLimit : ENV.APP.initialJobLimit,
  refresh : {
    enabled : true,
    interval: ENV.APP.refreshInterval
  },
  seconds : null,

  setupController : function (controller, model) {
    this._super(controller, model);
    controller.set('range', this.get('range'));
  },

  model: function () {
    this.set('seconds', 0);
    return this.store.filter('job', {
      limit: this.get('jobLimit')
    }, function () {
      return true;
    });
  },

  activate : function () {
    this.set('refresh.enabled', true);
  },

  deactivate : function () {
    this.set('refresh.enabled', false);
  },

  tick: function () {
    if (!this.get('refresh.enabled')) {
      return;
    }

    var route = this;
    var refreshIn = this.get('refresh.interval') * 1000;

    Ember.run.later(function () {
      var seconds = route.get('seconds');
      route.store.find('job', {
        ago: route.get('range')
      }).then(function () {
        route.set('seconds', seconds + refreshIn);
      });
    }, refreshIn);
  }.observes('seconds', 'refresh.enabled')

});