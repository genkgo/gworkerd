FROM rabbitmq

MAINTAINER Frederik Bosch <f.bosch@genkgo.nl>

RUN rabbitmq-plugins enable rabbitmq_web_stomp rabbitmq_stomp rabbitmq_management

RUN service rabbitmq-server restart

EXPOSE 15674

