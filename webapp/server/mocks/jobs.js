var jobs = [
  {
    id: '703b0151-c09b-4d86-8925-2219f17407fd',
    command: '/usr/bin/php test.php',
    exitCode: 200,
    startedAt: new Date(2015, 3, 9, 8, 0, 0),
    finishedAt: new Date(2015, 3, 9, 8, 10, 0),
    stdout: '[PHP] success',
    stderr: '[PHP] error'
  },
  {
    id: '65673aae-6caf-48c2-a9f2-31b001504a01',
    command: '/usr/bin/php test.php',
    exitCode: 200,
    startedAt: new Date(2015, 3, 9, 9, 0, 0),
    finishedAt: new Date(2015, 3, 9, 9, 10, 0),
    stdout: '[PHP] success',
    stderr: '[PHP] error'
  },
  {
    id: 'd87406a5-27b8-40a5-a094-fabddfd37b05',
    command: '/usr/bin/php test.php',
    exitCode: 200,
    startedAt: new Date(2015, 3, 9, 10, 0, 0),
    finishedAt: new Date(2015, 3, 9, 10, 10, 0),
    stdout: '[PHP] success',
    stderr: '[PHP] error'
  }
];

module.exports = function(app, wss) {
  var express = require('express');
  var jobsRouter = express.Router();
  var counter = 0;
  var limit = 1;

  jobsRouter.get('/', function(req, res) {
    var filtered = jobs;
    for (var index in req.query) {
      if (index === "limit") {
        continue;
      }
      filtered = filtered.filterBy(index, req.query[index]);
    }

    if (req.query["limit"]) {
      limit = req.query["limit"];
    }

    filtered = filtered.slice(0, counter + limit);
    counter++;

    res.send({
      'job': filtered,
      'meta': {
        'total' : jobs.length
      }
    });
  });

  jobsRouter.post('/', function(req, res) {
    res.status(201).end();
  });

  jobsRouter.get('/:id', function(req, res) {
    res.send({
      'job': jobs.find(function(job) {
        return job.id == req.params.id
      })
    });
  });

  jobsRouter.put('/:id', function(req, res) {
    res.send({
      'job': {
        id: req.params.id
      }
    });
  });

  jobsRouter.delete('/:id', function(req, res) {
    res.status(204).end();
  });

  app.use('/api/jobs', jobsRouter);

  var uuid = require('node-uuid');
  var events = require('events');
  var eventEmitter = new events.EventEmitter();

  var startNewJob = function () {
    var job = {
      id: uuid.v1(),
      command: '/usr/bin/php test.php',
      'exitCode': null,
      'startedAt': new Date(),
      'finishedAt': null,
      'stdout': 'STDOUT output',
      'stderr': 'STDERR output'
    };

    jobs.push(job);
    return job;
  };

  setInterval(function () {
    var runningJob = startNewJob();
    eventEmitter.emit('jobStarted', runningJob);
    setTimeout(function () {
      runningJob.exitCode = Math.random() < 0.5 ? 0 : 1;
      runningJob.finishedAt = new Date();
      eventEmitter.emit('jobFinished', runningJob);
    }, 3000);
  }, 2000);

  wss.on('connection', function (wss) {
    var sendUpdate = function (job) {
      try {
        wss.send(
          JSON.stringify({
            job: job
          })
        );
      } catch (e) {
      }
    };
    eventEmitter.on('jobFinished', sendUpdate);
    eventEmitter.on('jobStarted', sendUpdate);
  });
};