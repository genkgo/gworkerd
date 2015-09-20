# Gworkerd

Single worker that processes cli jobs with a graphical user interface for feedback.

## Usage

### Start Message and Result Server

Server should accept the STOMP protocol. It has been tested with Rabbit MQ only. By using the
[Dockerfile](assets/docker/Dockerfile) in this repo you can start the tested server by the following commands.

```
$ docker build --rm -t grabbit .
$ docker run -d -p 61613:61613 -p 15672:15672 grabbit
```

At this moment only mysql is supported as a result backend. There are plans to support multiple databases. In order to
start with mysql, create a [table](assets/sql/mysql_worker_results.sql) where you can store the results.

### Build gworkerd

```
$ cd daemon && cargo build
```

### Create a configuration file

```json
{
    "threads": 10,
    "log": {
        "directory": "/var/log/gworkerd",
        "levels": "gworkerd=info,mysql=warn,stomp=warn"
    },
    "mysql": {
        "address": "127.0.0.1",
        "username": "root",
        "password": "",
        "database": "worker_results"
    },
    "stomp": {
        "address": "127.0.0.1",
        "port": 61613,
        "username": "guest",
        "password": "guest",
        "host": "/",
        "topic": "/queue/gworkerd",
        "prefetch_count": 1
    },
     "monitor": {
         "address": "localhost:3000",
         "webapp_path": "../webapp/dist",
         "websockets": false,
         "password": "random generated password"
     }
}
```

### Start gworkerd

The daemon requires one argument: the location of the configuration file.

```
$ ./gworkerd config.json
```

### Send messages to the queue

Send your to the message queue. The content type of a job should be  application/json and has the following signature.

```json
{
    "id": "d334e01e-14cf-4247-b39a-7c9f417ad64b",
    "command": "mysqldump --all-databases",
    "cwd": "/your/directory"
}
```

### Monitor what is going on in your browser

Move your browser to the address of the monitor and you can see exactly what is going on.

`http://localhost:3000/monitor/`


## Roadmap

* 0.1.0 ~~Design of api and graphical interface (webapp in Ember)~~
* 0.2.0 ~~Pull jobs from STOMP and process jobs concurrently~~
* 0.3.0 ~~Enable logging to file~~
* 0.4.0 ~~Add config file to application~~
* 0.5.0 Connect daemon to webapp
* 0.6.0 Support websocket in daemon
* 0.7.0 Support multiple result backends and consumers
* 0.8.0 Retry jobs
* 0.9.0 Breaking backward compatibility changes
* 1.0.0 General Availabililty