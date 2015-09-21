import Ember from 'ember';

export default Ember.Controller.extend({
  queryParams: ['status'],
  reverse: function(){
    return this.get('model').toArray().reverse();
  }.property('model.[]')
});