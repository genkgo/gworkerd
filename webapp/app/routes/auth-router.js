import Ember from 'ember';

export default Ember.Route.extend({

  beforeModel: function () {
    if (this.controllerFor('application').get('login') === false) {
      this.transitionTo('login');
    }
  }
});