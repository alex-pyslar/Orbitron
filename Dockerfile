FROM ubuntu:latest
LABEL authors="alex_pyslar"

ENTRYPOINT ["top", "-b"]