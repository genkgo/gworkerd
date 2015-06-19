import Ember from 'ember';
import DS from 'ember-data';

export default DS.Model.extend({
  exitCode : DS.attr('number'),
  startedAt: DS.attr('date'),
  finishedAt: DS.attr('date'),
  stdout : DS.attr('string'),
  stderr : DS.attr('string'),
  duration: Ember.computed('executedAt', 'finishedAt', function () {
    var finishedAt = this.get('finishedAt');
    if (finishedAt === null) {
      return null;
    } else {
      var startedAt = this.get('startedAt');
      return moment(finishedAt).diff(moment(startedAt), 'seconds');
    }
  }),
  successful : Ember.computed('exitCode', function () {
    return this.get('exitCode') === 0;
  }),
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