import Ember from 'ember';
import ENV from 'gworkerd/config/environment';

export default Ember.Controller.extend({
  socketService: Ember.inject.service('websockets'),
  connected: false,
  login: window.sessionStorage.getItem('password') !== null,
  password: null,
  server: null,

  shouldStartSocket : Ember.observer('model', function () {
    if (this.get('server') && this.get('server').websockets) {
      var controller = this;
      var socket = this.get('socketService').socketFor(ENV.APP.data.socket);

      socket.on('open', function () {
        controller.set('connected', true);
      }, this);

      socket.on('close', function() {
        controller.set('connected', true);
        Ember.run.later(this, function() {
          socket.reconnect();
        }, 1000);
      }, this);

      socket.on('message', function (event) {
        var message = JSON.parse(event.data);
        controller.send('jobUpdate', message.job);
      }, this);
    }
  }).observes('server'),

  close : function () {
    this.get('socketService').closeSocketFor(ENV.APP.data.socket);
  }.on('willDestroy')

});