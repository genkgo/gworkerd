import AuthRouter from 'gworkerd/routes/auth-router';
import ENV from 'gworkerd/config/environment';

export default AuthRouter.extend({
  jobLimit : ENV.APP.initialJobLimit,

  model: function () {
    return this.store.filter('job', {
      limit: this.get('jobLimit')
    }, function () {
      return true;
    });
  }

});