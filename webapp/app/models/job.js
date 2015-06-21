import Ember from 'ember';
import DS from 'ember-data';

export default DS.Model.extend({
  command : DS.attr('string'),
  nicer : DS.attr('number'),
  attempts : DS.attr('number'),
  status : DS.attr('string'),
  results : DS.hasMany('result', { async : true }),
  lastTriedAt : DS.attr('date')
});