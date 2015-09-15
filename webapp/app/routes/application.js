import Ember from 'ember';

export default Ember.Route.extend({
  model: function () {
    return new Ember.RSVP.Promise((resolve) => {
      Ember.$.get('/api/server').then((request) => {
        resolve(request);
      });
    });
  },

  actions : {
    jobUpdate : function (job) {
      this.store.pushPayload({
        jobs: [job]
      });
    }
  }
});