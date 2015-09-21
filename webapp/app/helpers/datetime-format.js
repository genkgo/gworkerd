import Ember from 'ember';
import moment from 'moment';

export default Ember.Helper.helper(function(params) {
  return moment(params[0]).format('LLL');
});