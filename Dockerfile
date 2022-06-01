FROM public.ecr.aws/prima/rust:1.61.0-1

WORKDIR /code

RUN curl -slO https://get.helm.sh/helm-v3.3.4-linux-amd64.tar.gz && \
    tar -zxvf helm-v3.3.4-linux-amd64.tar.gz && \
    mv linux-amd64/helm /usr/local/bin/helm 

RUN chown -R app:app /code

# Needed to have the same file owner in the container and in Linux host
USER app
