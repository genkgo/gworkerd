import Ember from 'ember';
import DS from 'ember-data';

export default DS.Model.extend({
  command : DS.attr('string'),
  nicer : DS.attr('number'),
  attempts : DS.attr('number'),
  successful : DS.attr('boolean'),
  results : DS.hasMany('result', { async : true }),
  lastTriedAt : DS.attr('date'),

  status: Ember.computed('successful', function () {
    var success = this.get('successful');

    if (success === null) {
      return 'busy';
    }

    if (success === true) {
      return 'successful';
    }

    return 'failed';
  })
});