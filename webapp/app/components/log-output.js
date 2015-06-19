import Ember from 'ember';

export default Ember.Component.extend({
  displayed : false,

  actions : {
    show : function () {
      this.set('displayed', true);
    },
    close : function () {
      this.set('displayed', false);
    }
  }
});