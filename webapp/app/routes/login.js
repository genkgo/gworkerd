import Ember from 'ember';

export default Ember.Route.extend({

  actions: {
    submit: function (password) {
      Ember.$.post('/api/auth', {
        password: password
      }).done(() => {
        this.controllerFor('application').set('login', true);
        this.transitionTo('index');
        window.sessionStorage.setItem('password', password);
      }).fail(() => {
        this.controllerFor('login').set('failed', true);
        Ember.run.later(() => {
          this.controllerFor('login').set('failed', false);
        }, 3000);
      });
    }
  }

});