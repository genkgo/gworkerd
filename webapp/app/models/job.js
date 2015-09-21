import Ember from 'ember';
import DS from 'ember-data';
import moment from 'moment';

export default DS.Model.extend({
  command : DS.attr('string'),
  cwd : DS.attr('string'),
  status : DS.attr('number'),
  startedAt: DS.attr('date'),
  finishedAt: DS.attr('date'),
  stdout : DS.attr('string'),
  stderr : DS.attr('string'),

  duration: Ember.computed('startedAt', 'finishedAt', function () {
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

  readyState: Ember.computed('status', function () {
    var exitCode = this.get('status');

    if (exitCode === null) {
      return 'busy';
    }

    if (exitCode === 0) {
      return 'successful';
    }

    return 'failed';
  }),

  isFinished: Ember.computed('status', function () {
    return this.get('status') !== null;
  })
});