import Ember from 'ember';

export default Ember.Controller.extend({
  reverse: function(){
    return this.get('model').toArray().reverse();
  }.property('model.[]')
});