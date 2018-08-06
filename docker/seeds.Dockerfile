FROM postgres:9.5-alpine

WORKDIR /root

COPY ./seeds ./seeds

ENTRYPOINT ["psql"]

CMD ["--help"]
